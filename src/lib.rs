//! A very simple (slightly bad) memory allocator writen in rust.
//!
//!```rust
//!extern crate zack_alloc;
//!use zack_alloc::ZackAlloc;
//!
//!#[global_allocator]
//!static _A: ZackAlloc = ZackAlloc::new(); 
//!
//!fn main()
//!{
//!    let vec = vec![0i32; 10 * 1024]; // 40 kB
//!    
//!    let boxed_val = Box::new(5i32);
//!    
//!    let vec2 = vec![0i32; 10 * 1024]; // 40 kB
//! 
//!    drop(boxed_val);
//! 
//!    let boxed_val_2 = Box::new(10i32);
//! 
//!    println!("{:?}", vec);
//!}
//!```


use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr::null_mut;
use core::cell::RefCell;
//use std::sync::Mutex;

const WSIZE: usize = 4;
const CHUNK_SIZE: usize = (1 << 12);
const MAX_HEAP: usize = (200*(1<<20));  /* 200 MB */

#[inline]
fn pack(size: u32, alloc: bool) -> u32
{
    let alloc_val = alloc as u32;
    (size & !0x7) | (alloc_val) // size is assumed to be divisable by 8
    // in rust ! is a bitwise not
}

#[inline]
fn unpack(val: u32) -> (u32, bool)
{
    let alloc = val & 0x1 == 0x1;
    (val & !0x7, alloc)
}

#[inline]
unsafe fn get(p: *mut u8) -> u32 { *(p as *mut u32) }

#[inline]
unsafe fn put(p: *mut u8, val: u32) { *(p as *mut u32) = val; }

#[inline]
fn hdr_p(p: *mut u8) -> *mut u8{ (p as usize - WSIZE) as *mut u8 }

#[inline]
unsafe fn ftr_p(p: *mut u8) -> *mut u8{ (p as usize + unpack(get(hdr_p(p))).0 as usize - 2*WSIZE) as *mut u8 }

#[inline]
unsafe fn next_block_p(p: *mut u8) -> *mut u8{ (p as usize + unpack(get(hdr_p(p))).0 as usize) as *mut u8 }

#[inline]
unsafe fn prev_block_p(p: *mut u8) -> *mut u8
{ 
    let new_p = (p as usize - 2 * WSIZE) as *mut u8;
    (p as usize - unpack(get(new_p)).0 as usize) as *mut u8
}


pub struct ZackAlloc // This is not thread safe
{
    inner: RefCell<Option<ZackAllocInner>>,  // This is not thread safe
    len: usize
}

unsafe impl Sync for ZackAlloc {} // I just want to get around the thread safe error
                                  // I'll try to get rid of this in the future


impl ZackAlloc
{
    pub const fn new() -> Self
    {
        ZackAlloc {inner: RefCell::new(None), len: MAX_HEAP}
    }
}


struct ZackAllocInner
{
    mem_start_brk: *mut u8,
    mem_brk: *mut u8,
    mem_max_addr: *mut u8,

    heap_listp: *mut u8
}

impl ZackAllocInner
{
    unsafe fn mem_sbrk(&mut self, incr: usize) -> *mut u8 // TODO: Option
    {
        let old_brk = self.mem_brk;
        self.mem_brk = (self.mem_brk as usize + incr) as *mut u8;

        if self.mem_brk as usize > self.mem_max_addr as usize
        {
            panic!("Can't get more heap")
        }
        return old_brk;
    }

    unsafe fn mem_init(&mut self, len: usize)  // TODO: Option
    {
        /* allocate the storage we will use to model the available VM */
        self.mem_start_brk = System.alloc(Layout::from_size_align_unchecked(len, 4)); // Big cheat

        if self.mem_start_brk.is_null()
        {
            panic!("IDK");
        }

        self.mem_max_addr = (self.mem_start_brk as usize + len) as *mut u8;  /* max legal heap address */
        self.mem_brk = self.mem_start_brk; 
    }
}

impl ZackAllocInner
{
    pub fn new(len: usize) -> Self
    {
        let mut ret = ZackAllocInner { mem_start_brk: null_mut(), mem_brk: null_mut(), mem_max_addr: null_mut(), heap_listp: null_mut() };

        unsafe { 
            ret.mem_init(len);
            ret.heap_listp = ret.mem_sbrk(4*WSIZE);

            put(ret.heap_listp, pack(0,false)); //alignment so that we can have heap_listp be at the prologue footer
            put((ret.heap_listp as usize + 1*WSIZE) as *mut u8, pack(2*WSIZE as u32,true)); // Prologue header
            // we want the pointer for heap_listp to be here
            put((ret.heap_listp as usize + 2*WSIZE) as *mut u8, pack(2*WSIZE as u32,true)); // prologue footer
            put((ret.heap_listp as usize + 3*WSIZE) as *mut u8, pack(0,true)); // Epilogue block

            ret.heap_listp = (ret.heap_listp as usize + 2*WSIZE) as *mut u8; //8 bytes because heap_listp is a char* then we inc by bytes

            ret.extend_heap(CHUNK_SIZE/WSIZE); //extend_heap takes words and CHUNK_SIZE is in bytes
        }

        ret
    }

    unsafe fn extend_heap(&mut self, words: usize) -> *mut u8 // TODO: Option
    {
        let size = if words%2 == 0 { words } else { words+1 } * WSIZE;

        let new_block_p = self.mem_sbrk(size); // points to after the original epilogue block

        put(hdr_p(new_block_p), pack(size as u32, false));
        put(ftr_p(new_block_p), pack(size as u32, false));

        put(hdr_p(next_block_p(new_block_p)), pack(0,true));// we need to add back the Epilogue block

        //we need to coalesce in the case that the previus block is not allocated
        let new_coalesced_block_p = self.coalesce(new_block_p);

        new_coalesced_block_p
    }

    unsafe fn coalesce(&self, bp: *mut u8) -> *mut u8
    {
        // If bp is unalloc check the block infront and behind.
        // if the one infront is unalloc changes it to be bp and change bp size to add the old bp size
        // if the one in back is unalloc change bps size to add the new

        let bp_size = unpack(get(hdr_p(bp))).0;
        let prev_is_alloc = unpack(get(ftr_p(prev_block_p(bp)))).1;
        let next_is_alloc = unpack(get(hdr_p(next_block_p(bp)))).1;

        if prev_is_alloc && next_is_alloc
        {
            bp
        }
        else if prev_is_alloc && !next_is_alloc // add next block
        {
            let new_bp_size = bp_size + unpack(get(hdr_p(next_block_p(bp)))).0;

            put(hdr_p(bp), pack(new_bp_size, false));
            put(ftr_p(bp), pack(new_bp_size, false));
            bp
        }
        else if !prev_is_alloc && next_is_alloc
        {
            let new_bp_size = bp_size + unpack(get(hdr_p(prev_block_p(bp)))).0;

            put(ftr_p(bp), pack(new_bp_size, false));
            put(hdr_p(prev_block_p(bp)), pack(new_bp_size, false));

            prev_block_p(bp)
        }
        else
        {
            let new_bp_size = bp_size + unpack(get(hdr_p(prev_block_p(bp)))).0 + unpack(get(hdr_p(next_block_p(bp)))).0;

            put(hdr_p(prev_block_p(bp)), pack(new_bp_size, false));
            put(ftr_p(next_block_p(bp)), pack(new_bp_size, false));

            prev_block_p(bp)
        }
    }

    unsafe fn find_fit(&self, size: usize) -> Option<*mut u8>
    {
        let mut this_block_p = self.heap_listp;
        
        while unpack(get(hdr_p(this_block_p))).0 != 0 // TODO: turn blocks into link list and add iter
        {
            let block_data = unpack(get(hdr_p(this_block_p)));
            if !block_data.1 && block_data.0 as usize >= size
            {
                return Some(this_block_p);
            }

            this_block_p = next_block_p(this_block_p);
        }

        return None;
    }

    unsafe fn place(&self, ptr: *mut u8, size: usize) -> *mut u8
    {
        let size = size as u32;
        let full_block_size = unpack(get(hdr_p(ptr))).0;
        let leftover_size = full_block_size - size;
        
        // TODO: what if size > full_block_size

        if leftover_size < 4*WSIZE as u32
        {
            put(hdr_p(ptr), pack(full_block_size, true));
            put(ftr_p(ptr), pack(full_block_size, true));
        }
        else
        {
            put(hdr_p(ptr), pack(size, true));
            put(ftr_p(ptr), pack(size, true));

            let next_blk_p = next_block_p(ptr);

            put(hdr_p(next_blk_p), pack(leftover_size, false));
            put(ftr_p(next_blk_p), pack(leftover_size, false));
        }

        ptr
    }

    unsafe fn mm_malloc(&mut self, size: usize) -> *mut u8
    {
        let size_padded = if size <= 2 * WSIZE {4 * WSIZE} else { (((size - 1)/8) + 2) * 8 }; //round up to the nearist 8 bytes then add 8 (2 * WSIZE)

        if let Some(bp) = self.find_fit(size_padded)
        {
            self.place(bp, size_padded)
        }
        else
        {
            let ext_size = if size_padded > CHUNK_SIZE {size_padded} else {CHUNK_SIZE};

            let bp = self.extend_heap(ext_size / WSIZE);

            self.place(bp, size_padded)
        }
    }

    unsafe fn mm_free(&mut self, ptr: *mut u8) -> ()
    {
        let size = unpack(get(hdr_p(ptr))).0;

        put(hdr_p(ptr), pack(size, false));
        put(ftr_p(ptr), pack(size, false));

        self.coalesce(ptr);
    }
}

unsafe impl GlobalAlloc for ZackAlloc
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8
    {
        let mut inner = self.inner.borrow_mut();

        if inner.is_none()
        {
            inner.replace(ZackAllocInner::new(self.len));
        }
        
        inner.as_mut().unwrap().mm_malloc(layout.size())
        //System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout)
    {
        let mut inner = self.inner.borrow_mut();

        if inner.is_none()
        {
            inner.replace(ZackAllocInner::new(self.len));
        }
        
        inner.as_mut().unwrap().mm_free(ptr)
        //System.dealloc(ptr, layout)
    }
}
