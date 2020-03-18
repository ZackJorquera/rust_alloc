extern crate zack_alloc;
use zack_alloc::ZackAlloc;

#[global_allocator]
static A: ZackAlloc = ZackAlloc::new();


fn main()
{
    test();
}

fn test()
{
    let vec = vec![0i32; 10 * 1024]; // 40 kB
    
    let boxed_val_3 = Box::new(0i32);
    let box_ptr_3 = Box::into_raw(boxed_val_3);
    let boxed_val_3 = unsafe { Box::from_raw(box_ptr_3) };
    
    let boxed_val = Box::new(5i32);
    let box_ptr = Box::into_raw(boxed_val);
    let boxed_val = unsafe { Box::from_raw(box_ptr) };
    
    let vec2 = vec![0i32; 10 * 1024]; // 40 kB
    drop(boxed_val);
    
    let boxed_val_2 = Box::new(5i32);
    let box_ptr_2 = Box::into_raw(boxed_val_2);
    let boxed_val_2 = unsafe { Box::from_raw(box_ptr_2) };
    drop(boxed_val_2);
    
    drop(boxed_val_3);
    
    
    println!("{}", vec[1000]); // make sure the vectors were compiled
    println!("{}", vec2[1000]);
    println!("{:#x?} {:#x?}", box_ptr, box_ptr_2);

    assert_eq!(box_ptr, box_ptr_2);
    assert_ne!(box_ptr, box_ptr_3);
    assert_ne!(box_ptr_2, box_ptr_3);

}

#[test] // I can't see to get this to work
fn test_1()
{
    let vec = vec![0i32; 10 * 1024]; // 40 kB
    
    let boxed_val_3 = Box::new(0i32);
    let box_ptr_3 = Box::into_raw(boxed_val_3);
    let boxed_val_3 = unsafe { Box::from_raw(box_ptr_3) };
    
    let boxed_val = Box::new(5i32);
    let box_ptr = Box::into_raw(boxed_val);
    let boxed_val = unsafe { Box::from_raw(box_ptr) };
    
    let vec2 = vec![0i32; 10 * 1024]; // 40 kB
    drop(boxed_val);
    
    let boxed_val_2 = Box::new(5i32);
    let box_ptr_2 = Box::into_raw(boxed_val_2);
    let boxed_val_2 = unsafe { Box::from_raw(box_ptr_2) };
    drop(boxed_val_2);
    
    drop(boxed_val_3);
    
    
    println!("{}", vec[1000]); // make sure the vectors were compiled
    println!("{}", vec2[1000]);
    println!("{:#x?} {:#x?}", box_ptr, box_ptr_2);

    assert_eq!(box_ptr, box_ptr_2);
    assert_ne!(box_ptr, box_ptr_3);
    assert_ne!(box_ptr_2, box_ptr_3);
}