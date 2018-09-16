// mod cl_impl
// just here to make the main function look cleaner
extern crate ocl;

use super::{WIDTH, HEIGHT};

pub struct CL {
    kernel:     ocl::Kernel,
    buffer_in:  ocl::Buffer<u8>,
    buffer_out: ocl::Buffer<u8>,
}

impl CL {
    pub fn new(kernel_source: &str) -> ocl::Result<CL> {
        let pro_que = ocl::ProQue::builder()
                                   .src(kernel_source)
                                   .dims((WIDTH, HEIGHT))
                                   .build()?;
        let buffer_in  = pro_que.create_buffer::<u8>()?;
        let buffer_out = pro_que.create_buffer::<u8>()?;
        let kernel     = pro_que.kernel_builder("life")
                                .arg(&buffer_in)
                                .arg(&buffer_out)
                                .build()?;
        let cl = CL {
            kernel,
            buffer_in,
            buffer_out,
        };

        Ok(cl)
    }

    pub fn write(&self, data: &[u8]) -> ocl::Result<()> {
        self.buffer_in.write(data).enq()
    }

    pub fn read(&self, data: &mut [u8]) -> ocl::Result<()> {
        self.buffer_out.read(data).enq()
    }

    pub fn enq_kernel(&self) -> ocl::Result<()> {
        unsafe {
            self.kernel.enq()
        }
    }
}
