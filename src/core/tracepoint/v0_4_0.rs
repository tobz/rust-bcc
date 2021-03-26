use bcc_sys::bccapi::*;

use crate::helpers::to_cstring;
use crate::types::MutPointer;
use crate::BccError;

use std::ffi::CString;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::os::unix::prelude::*;
use std::ptr;

#[derive(Debug)]
pub struct Tracepoint {
    subsys: CString,
    name: CString,
    code_fd: File,
    p: MutPointer,
}

impl Tracepoint {
    pub fn attach_tracepoint(subsys: &str, name: &str, file: File) -> Result<Self, BccError> {
        let cname = to_cstring(name, "name")?;
        let csubsys = to_cstring(subsys, "subsys")?;
        // NOTE: BPF events are system-wide and do not support CPU filter
        let (pid, cpu, group_fd) = (-1, 0, -1);
        let ptr = unsafe {
            bpf_attach_tracepoint(
                file.as_raw_fd(),
                csubsys.as_ptr(),
                cname.as_ptr(),
                pid,
                cpu,
                group_fd,
                None,
                ptr::null_mut(),
            )
        };
        if ptr.is_null() {
            Err(BccError::AttachTracepoint {
                subsys: subsys.to_string(),
                name: name.to_string(),
            })
        } else {
            Ok(Self {
                subsys: csubsys,
                name: cname,
                code_fd: file,
                p: ptr,
            })
        }
    }
}

impl PartialEq for Tracepoint {
    fn eq(&self, other: &Tracepoint) -> bool {
        self.subsys == other.subsys && self.name == other.name
    }
}

impl Eq for Tracepoint {}

impl Hash for Tracepoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.subsys.hash(state);
        self.name.hash(state);
    }
}

impl Drop for Tracepoint {
    fn drop(&mut self) {
        unsafe {
            bpf_detach_tracepoint(self.subsys.as_ptr(), self.name.as_ptr());
        }
    }
}
