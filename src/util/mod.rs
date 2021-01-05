use spin::Mutex;

//pub mod mmio;

pub unsafe fn get_mutex_mut<T>(m: &Mutex<T>) -> &mut T {
    (&mut *(m as *const _ as *mut Mutex<T>)).get_mut()
}
