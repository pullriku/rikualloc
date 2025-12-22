use rikualloc::{
    allocator::bump::BumpAllocator, mutex::Locked,
    source::static_buff::StaticBuffer,
};

static BUFFER: Locked<StaticBuffer<{ 1024 * 1024 }>> =
    Locked::new(StaticBuffer::new());

#[global_allocator]
static BUMP: Locked<BumpAllocator<&Locked<StaticBuffer<{ 1024 * 1024 }>>>> =
    Locked::new(BumpAllocator::new(&BUFFER));

fn main() {
    let vec: Vec<usize> = (0..10).filter(|x| x % 13 == 0).collect();

    println!("{vec:?}");
}
