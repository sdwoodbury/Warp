use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use crate::tesseract::Tesseract;

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_new() -> *mut Tesseract {
    Box::into_raw(Box::new(Tesseract::default())) as *mut Tesseract
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_from_file(file: *const c_char) -> *mut Tesseract {
    if file.is_null() {
        return std::ptr::null_mut();
    }

    let cname = CStr::from_ptr(file).to_string_lossy().to_string();
    match Tesseract::from_file(cname) {
        Ok(tesseract) => Box::into_raw(Box::new(tesseract)) as *mut Tesseract,
        Err(_) => std::ptr::null_mut(),
    }
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_to_file(tesseract: *mut Tesseract, file: *const c_char) -> bool {
    if tesseract.is_null() {
        return false;
    }

    if file.is_null() {
        return false;
    }

    let tesseract = &mut *tesseract;
    let cname = CStr::from_ptr(file).to_string_lossy().to_string();
    tesseract.to_file(cname).is_ok()
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_set_file(
    tesseract: *mut Tesseract,
    file: *const c_char,
) -> bool {
    if tesseract.is_null() {
        return false;
    }

    if file.is_null() {
        return false;
    }

    let tesseract = &mut *tesseract;
    let cname = CStr::from_ptr(file).to_string_lossy().to_string();
    tesseract.set_file(cname);
    true
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_set_autosave(tesseract: *mut Tesseract) -> bool {
    if tesseract.is_null() {
        return false;
    }

    let tesseract = &mut *tesseract;
    tesseract.set_autosave();
    true
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_autosave_enabled(tesseract: *mut Tesseract) -> bool {
    if tesseract.is_null() {
        return false;
    }

    let tesseract = &*tesseract;
    tesseract.autosave_enabled()
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_save(tesseract: *mut Tesseract) -> bool {
    if tesseract.is_null() {
        return false;
    }

    let tesseract = &mut *tesseract;
    tesseract.save().is_ok()
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_set(
    tesseract: *mut Tesseract,
    key: *const c_char,
    val: *const c_char,
) -> bool {
    if tesseract.is_null() {
        return false;
    }
    if key.is_null() {
        return false;
    }
    if val.is_null() {
        return false;
    }

    let tesseract = &mut *tesseract;
    let c_key = CStr::from_ptr(key).to_string_lossy().to_string();
    let c_val = CStr::from_ptr(val).to_string_lossy().to_string();

    tesseract.set(&c_key, &c_val).is_ok()
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_retrieve(
    tesseract: *mut Tesseract,
    key: *const c_char,
) -> *mut c_char {
    if tesseract.is_null() {
        return std::ptr::null_mut();
    }
    if key.is_null() {
        return std::ptr::null_mut();
    }

    let tesseract = &mut *tesseract;
    let c_key = CStr::from_ptr(key).to_string_lossy().to_string();

    match tesseract.retrieve(&c_key) {
        Ok(val) => match CString::new(val) {
            Ok(c) => c.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_exist(tesseract: *mut Tesseract, key: *const c_char) -> bool {
    if tesseract.is_null() {
        return false;
    }
    if key.is_null() {
        return false;
    }

    let tesseract = &*tesseract;
    let c_key = CStr::from_ptr(key).to_string_lossy().to_string();
    tesseract.exist(&c_key)
}
#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_delete(tesseract: *mut Tesseract, key: *const c_char) -> bool {
    if tesseract.is_null() {
        return false;
    }
    if key.is_null() {
        return false;
    }

    let tesseract = &mut *tesseract;
    let c_key = CStr::from_ptr(key).to_string_lossy().to_string();
    tesseract.delete(&c_key).is_ok()
}
#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_clear(tesseract: *mut Tesseract) {
    if tesseract.is_null() {
        return;
    }

    let tesseract = &mut *tesseract;
    tesseract.clear()
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_is_unlock(tesseract: *mut Tesseract) -> bool {
    if tesseract.is_null() {
        return false;
    }

    let tesseract = &mut *tesseract;
    tesseract.is_unlock()
}

//TODO: Have key be bytes
#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_unlock(tesseract: *mut Tesseract, key: *const c_char) -> bool {
    if tesseract.is_null() {
        return false;
    }

    if key.is_null() {
        return false;
    }

    let tesseract = &mut *tesseract;
    let c_key = CStr::from_ptr(key).to_string_lossy().to_string();
    tesseract.unlock(c_key.as_bytes()).is_ok()
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_lock(tesseract: *mut Tesseract) -> bool {
    if tesseract.is_null() {
        return false;
    }

    let tesseract = &mut *tesseract;
    tesseract.lock();
    true
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn tesseract_free(tesseract: *mut Tesseract) {
    if tesseract.is_null() {
        return;
    }
    drop(Box::from_raw(tesseract))
}
