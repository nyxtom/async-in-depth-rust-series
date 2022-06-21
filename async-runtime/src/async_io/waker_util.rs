use std::{
    mem::ManuallyDrop,
    sync::Arc,
    task::{RawWaker, RawWakerVTable, Waker},
};

pub fn waker_fn<F: Fn() + 'static>(f: F) -> Waker {
    let raw = Arc::into_raw(Arc::new(f)) as *const ();
    let vtable = &WakerHelper::<F>::VTABLE;
    unsafe { Waker::from_raw(RawWaker::new(raw, vtable)) }
}

struct WakerHelper<F>(F);

impl<F: Fn() + 'static> WakerHelper<F> {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_waker,
        Self::wake,
        Self::wake_by_ref,
        Self::drop_waker,
    );

    unsafe fn clone_waker(ptr: *const ()) -> RawWaker {
        let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const F));
        std::mem::forget(arc.clone());
        RawWaker::new(ptr, &Self::VTABLE)
    }

    unsafe fn wake(ptr: *const ()) {
        let arc = Arc::from_raw(ptr as *const F);
        (arc)();
    }

    unsafe fn wake_by_ref(ptr: *const ()) {
        let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const F));
        (arc)();
    }

    unsafe fn drop_waker(ptr: *const ()) {
        drop(Arc::from_raw(ptr as *const F));
    }
}
