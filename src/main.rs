use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr::null_mut;
use core::cell::RefCell;
use std::sync::Mutex;

struct ZackAlloc
{
    inner: RefCell<Option<ZackAllocInner>>,
    len: usize
}

unsafe impl Sync for ZackAlloc {} // I just want to get around the thread safe error
                                  // I'll try to get rid of this in the future

impl ZackAlloc
{
    pub const fn new(len: usize) -> Self
    {
        ZackAlloc {inner: RefCell::new(None), len: len}
    }
}

struct ZackAllocInner
{
    mem_start_brk: *mut u8,
    mem_brk: *mut u8,
    mem_max_addr: *mut u8
}

impl ZackAllocInner
{
    pub fn new(len: usize) -> Self
    {
        let mut ret = ZackAllocInner { mem_start_brk: null_mut(), mem_brk: null_mut(), mem_max_addr: null_mut() };
        //printf("in mm_init\n");
        unsafe { ret.mem_init(len); }
        //heap_listp = mem_sbrk(4*WSIZE);

        //PUT(heap_listp + 0*WSIZE, 0); //alignment so that we can have heap_listp be at the prologue footer
        //PUT(heap_listp + 1*WSIZE, PACK(DSIZE,1)); // Prologue header
        // we want the pointer for heap_listp to be here
        //PUT(heap_listp + 2*WSIZE, PACK(DSIZE,1)); // prologue footer
        //PUT(heap_listp + 3*WSIZE, PACK(0,1)); // Epilogue block

        //heap_listp += 2*WSIZE; //8 bytes because heap_listp is a char* then we inc by bytes

        //printf("1\n");

        //if(extend_heap(CHUNKSIZE/WSIZE) == NULL) //extend_heap takes words and CHUNKSIZE is in bytes
        //    return -1;
        //return 0;

        ret
    }

    unsafe fn mem_init(&mut self, len: usize)
    {
        /* allocate the storage we will use to model the available VM */
        self.mem_start_brk = System.alloc(Layout::from_size_align_unchecked(len, 4));

        if self.mem_start_brk.is_null()
        {
            panic!("IDK");
        }

        self.mem_max_addr = (self.mem_start_brk as usize + len) as *mut u8;  /* max legal heap address */
        self.mem_brk = self.mem_start_brk; 
    }

    unsafe fn mm_malloc(&mut self, size: usize) -> *mut u8
    {
        let cur_brk = self.mem_brk;
        if size > 64
        {
            self.mem_brk = (self.mem_brk as usize + 128) as *mut u8;
        }
        else if size > 16
        {
            self.mem_brk = (self.mem_brk as usize + 64) as *mut u8;
        }
        else
        {
            self.mem_brk = (self.mem_brk as usize + 16) as *mut u8;
        }

        cur_brk
    }

    unsafe fn mm_free(&mut self, ptr: *mut u8) -> ()
    {

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
        
        inner.as_mut().expect("nerver").mm_malloc(layout.size())
        //System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout)
    {
        let mut inner = self.inner.borrow_mut();

        if inner.is_none()
        {
            inner.replace(ZackAllocInner::new(self.len));
        }
        
        inner.as_mut().expect("nerver").mm_free(ptr)
        //System.dealloc(ptr, layout)
    }
}

#[cfg_attr(not(test), global_allocator)]
static A: ZackAlloc = ZackAlloc::new(1024);

fn main()
{
    //let float_test: f64 = 0.0;
    //let layout = Layout::for_value(&float_test);
    //println!("{:?} {} {}", layout, layout.size(), layout.align());
    

    //let mut vec = vec![1,2,3];
    //vec.push(4);
    //println!("{:?}", vec);

    let a = Box::new(15);
    println!("{}", a);
}