use crate::ringbuf_shared::*;
use std::io::Write;
use std::cell::RefCell;

thread_local! {
    pub static LOG: RefCell<Option<LogHandle>> = RefCell::new(None);
}

pub fn global_set_log(l: &mut Log) {
    let hdl = l.new_handle();
    LOG.with(move |f| {
        *f.borrow_mut() = Some(hdl);
    });
}

pub fn log<F: Fn(&mut std::io::BufWriter<&mut [u8]>)>(f: F) {
    use std::borrow::Borrow;

    LOG.with(|l| {
        let lh = l.borrow_mut();
        if let Some(lh) = (*(*lh.borrow())).as_ref() {
            lh.log(f);
        }
    });
}

struct LogEntry {
    buf: [u8; 128],
}

pub struct Log {
    rb: RingBuf<LogEntry>,
    th: Option<std::thread::JoinHandle<()>>,
}

pub struct LogHandle {
    rb: RingBuf<LogEntry>,
}

impl LogHandle {
    pub fn log_str(&self, s: &str) {
        let mut ent = LogEntry { buf: [0; 128] };
        {
            let mut bw = std::io::BufWriter::new(&mut ent.buf[..]);
            use std::io::Write;
            write!(bw, "{}", s).unwrap();
        }
        self.rb.push(ent);
    }

    pub fn log<F: Fn(&mut std::io::BufWriter<&mut [u8]>)>(&self, f: F) {
        let mut ent = LogEntry { buf: [0; 128] };
        {
            let mut bw = std::io::BufWriter::new(&mut ent.buf[..]);
            f(&mut bw);
        }
        self.rb.push(ent);
    }
}

impl Log {
    pub fn new() -> Self {
        Self {
            rb: RingBuf::new(1024),
            th: None,
        }
    }

    pub fn log<F: Fn(&mut std::io::BufWriter<&mut [u8]>)>(&self, f: F) {
        let mut ent = LogEntry { buf: [0; 128] };
        {
            let mut bw = std::io::BufWriter::new(&mut ent.buf[..]);
            f(&mut bw);
        }
        self.rb.push(ent);
    }

    pub fn new_handle(&self) -> LogHandle {
        LogHandle { rb: self.rb.clone() }
    }

    pub fn collect(&self) -> Option<String> {
        let mut out_bytes = vec![];
        while let Some(le) = self.rb.pop() {
            let mut end = 0;
            for i in 0..le.buf.len() {
                if le.buf[i] == 0 {
                    end = i;
                    break;
                }
            }
            out_bytes.extend_from_slice(&le.buf[0..end]);
            out_bytes.extend_from_slice(b"\r\n");
        }
        if out_bytes.len() > 0 {
            Some(String::from_utf8_lossy(&out_bytes).to_string())
        } else {
            None
        }
    }

    pub fn start_writer_thread(&mut self) -> String {
        use std::fs::OpenOptions;

        {
            let mut file =
                OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open("/tmp/kickmess.log");
            let mut file =
                match file {
                    Ok(file) => file,
                    Err(e) => return format!("File open error: {}", e),
                };

            match file.write_all(b"--------------- START -----------\r\n") {
                Ok(_) => (),
                Err(e) => {
                    return format!("File open error: {}", e);
                },
            }
        }

        let rb = self.rb.clone();
        let th = std::thread::spawn(move || {
            std::thread::spawn(move || {
                let mut i = 0;
                loop {
                    i += 1;
                    let mut out_bytes = vec![];
                    while let Some(le) = rb.pop() {
                        let mut end = 0;
                        for i in 0..le.buf.len() {
                            if le.buf[i] == 0 {
                                end = i;
                                break;
                            }
                        }
                        out_bytes.extend_from_slice(&le.buf[0..end]);
                        out_bytes.extend_from_slice(b"\r\n");
                    }

                    if i % 10 == 0 {
                        out_bytes.extend_from_slice(b"TICK\r\n");
                    }

                    if out_bytes.len() > 0 {
                        let mut file =
                            OpenOptions::new()
                            .create(true)
                            .write(true)
                            .append(true)
                            .open("/tmp/kickmess.log").unwrap();

                        file.write_all(&out_bytes[..]).unwrap();
                        file.flush();
                    }

                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            })
        });

        th.join();

        String::from("ok")
    }
}
