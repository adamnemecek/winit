use crate::{
    platform::macos::ActivationPolicy,
    platform_impl::platform::{activation_hack, app_state::AppState},
};

use cocoa::base::id;
use objc::{
    declare::ClassDecl,
    runtime::{Class, Object, Sel},
};
use std::{
    cell::{RefCell, RefMut},
    os::raw::c_void,
};

static AUX_DELEGATE_STATE_NAME: &str = "auxState";

pub struct AuxDelegateState {
    /// We store this value in order to be able defer setting the activation policy until
    /// after the app has finished launching. If the activation policy is set earlier, the
    /// menubar is unresponsive for example.
    pub activation_policy: ActivationPolicy,
}

pub struct AppDelegateClass(pub *const Class);
unsafe impl Send for AppDelegateClass {}
unsafe impl Sync for AppDelegateClass {}

lazy_static! {
    pub static ref APP_DELEGATE_CLASS: AppDelegateClass = unsafe {
        let superclass = class!(NSResponder);
        let mut decl = ClassDecl::new("WinitAppDelegate", superclass).unwrap();

        decl.add_class_method(sel!(new), new as extern "C" fn(&Class, Sel) -> id);
        decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));
        decl.add_method(
            sel!(applicationDidFinishLaunching:),
            did_finish_launching as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(applicationDidBecomeActive:),
            did_become_active as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(applicationDidResignActive:),
            did_resign_active as extern "C" fn(&Object, Sel, id),
        );

        decl.add_ivar::<*mut c_void>(AUX_DELEGATE_STATE_NAME);
        decl.add_ivar::<*mut c_void>(activation_hack::State::name());
        decl.add_method(
            sel!(activationHackMouseMoved:),
            activation_hack::mouse_moved as extern "C" fn(&Object, Sel, id),
        );

        AppDelegateClass(decl.register())
    };
}

/// Safety: Assumes that Object is an instance of APP_DELEGATE_CLASS
pub unsafe fn get_aux_state_mut(this: &Object) -> RefMut<'_, AuxDelegateState> {
    let ptr: *mut c_void = *this.get_ivar(AUX_DELEGATE_STATE_NAME);
    // Watch out that this needs to be the correct type
    (*(ptr as *mut RefCell<AuxDelegateState>)).borrow_mut()
}

extern "C" fn new(class: &Class, _: Sel) -> id {
    unsafe {
        let this: id = msg_send![class, alloc];
        let this: id = msg_send![this, init];
        (*this).set_ivar(
            AUX_DELEGATE_STATE_NAME,
            Box::into_raw(Box::new(RefCell::new(AuxDelegateState {
                activation_policy: ActivationPolicy::Regular,
            }))) as *mut c_void,
        );
        (*this).set_ivar(
            activation_hack::State::name(),
            activation_hack::State::new(),
        );
        this
    }
}

extern "C" fn dealloc(this: &Object, _: Sel) {
    unsafe {
        activation_hack::State::free(activation_hack::State::get_ptr(this));
    }
}

extern "C" fn did_finish_launching(this: &Object, _: Sel, _: id) {
    trace!("Triggered `applicationDidFinishLaunching`");
    AppState::launched(this);
    trace!("Completed `applicationDidFinishLaunching`");
}

extern "C" fn did_become_active(this: &Object, _: Sel, _: id) {
    trace!("Triggered `applicationDidBecomeActive`");
    unsafe {
        activation_hack::State::set_activated(this, true);
    }
    trace!("Completed `applicationDidBecomeActive`");
}

extern "C" fn did_resign_active(this: &Object, _: Sel, _: id) {
    trace!("Triggered `applicationDidResignActive`");
    unsafe {
        activation_hack::refocus(this);
    }
    trace!("Completed `applicationDidResignActive`");
}
