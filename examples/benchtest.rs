//fn degain(i: &[f32], o: &mut [f32]) {
//    for (i, o) in i.iter().zip(o.iter_mut()) {
//        *o = *i * 0.5;
//    }
//}
//
//fn ingain(i: &[f32], o: &mut [f32]) {
//    for (i, o) in i.iter().zip(o.iter_mut()) {
//        *o = *i * 1.4;
//    }
//}
//
//fn distort(i: &[f32], o: &mut [f32]) {
//    for (i, o) in i.iter().zip(o.iter_mut()) {
//        if i.abs() >= 0.5 {
//            *o = -1.0 * *i;
//        }
//    }
//}
//
//const MAX_BUF_SIZE : usize = 64 * 2;
//
//trait OpFun {
//    fn name(&self) -> &str { "unknown" }
//    fn scalar_name(&self, idx: usize) -> &str { "?" }
//    fn scalar_count(&self) -> usize { 0 }
//    fn input_name(&self, idx: usize) -> &str { "?" }
//    fn input_count(&self) -> usize { 0 }
//    fn process(&mut self, scalars: &[f32], inputs: &[[f32; MAX_BUF_SIZE]], out: &mut [f32; MAX_BUF_SIZE]) {
//        if inputs.len() > 0 {
//            for (o, i) in out.iter_mut().zip(inputs[0].iter()) {
//                *o = *i;
//            }
//        } else {
//            for o in out.iter_mut() {
//                *o = 0;
//            }
//        }
//    }
//}
//
//struct Op<'a> {
//    input_scalar_idxs:  Vec<usize>,
//    input_op_idxs:      Vec<usize>,
//    out:                [f32; MAX_BUF_SIZE],
//    fun:                Box<dyn OpFun>,
//}
//
//struct Unit {
//    scalars: Vec<f32>,
//    ops:     Vec<Op>,
//}

enum TestMode {
    A,
    B,
    C,
    D,
    E
}

trait GGG {
    fn tick(&mut self, i: i64, out: &mut f64);
}

struct A;
impl GGG for A { fn tick(&mut self, i: i64, out: &mut f64) {
    *out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
} }
struct B;
impl GGG for B { fn tick(&mut self, i: i64, out: &mut f64) {
    *out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
} }
struct C;
impl GGG for C { fn tick(&mut self, i: i64, out: &mut f64) {
    *out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
    *out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
} }
struct D;
impl GGG for D { fn tick(&mut self, i: i64, out: &mut f64) {
    *out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
    *out -= ((i as f64) * std::f64::consts::PI / 1000.0).sin();
    *out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
} }
struct E;
impl GGG for E { fn tick(&mut self, i: i64, out: &mut f64) {
    *out -= ((i as f64) * std::f64::consts::PI / 1000.0).cos();
    *out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
    *out -= ((i as f64) * std::f64::consts::PI / 1000.0).cos();
} }


struct Test {
    mode: TestMode,
    fun: Box<Fn(i64, &mut f64)>,
    ft: Box<dyn GGG>,
}

impl Test {
    fn check(&mut self) {
        match self.mode {
            _ => {}
        }
    }

    fn init_match_check(&mut self) {
        match self.mode {
            TestMode::A => {
                self.ft = Box::new(A);
                self.fun = Box::new(|i: i64, out|{
                    *out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
                })
            },
            TestMode::B => {
                self.ft = Box::new(B);
                self.fun = Box::new(|i: i64, out|{
                    *out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                })
            },
            TestMode::C => {
                self.ft = Box::new(C);
                self.fun = Box::new(|i: i64, out|{
                    *out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                    *out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
                })
            },
            TestMode::D => {
                self.ft = Box::new(D);
                self.fun = Box::new(|i: i64, out|{
                    *out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                    *out -= ((i as f64) * std::f64::consts::PI / 1000.0).sin();
                    *out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                })
            },
            TestMode::E => {
                self.ft = Box::new(E);
                self.fun = Box::new(|i: i64, out|{
                    *out -= ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                    *out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
                    *out -= ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                })
            },
        }
    }

    fn a(&mut self, i: i64, out: &mut f64) {
        *out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
    }

    fn b(&mut self, i: i64, out: &mut f64) {
        *out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
    }

    fn c(&mut self, i: i64, out: &mut f64) {
        *out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
        *out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
    }

    fn d(&mut self, i: i64, out: &mut f64) {
        *out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
        *out -= ((i as f64) * std::f64::consts::PI / 1000.0).sin();
        *out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
    }

    fn e(&mut self, i: i64, out: &mut f64) {
        *out -= ((i as f64) * std::f64::consts::PI / 1000.0).cos();
        *out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
        *out -= ((i as f64) * std::f64::consts::PI / 1000.0).cos();
    }

    fn match_check_vt(&mut self) -> f64 {
        let mut out = 0.0;
        for i in 0..1000 {
            self.ft.tick(i, &mut out);
        }
        out
    }

    fn match_check_direct(&mut self) -> f64 {
        let mut out = 0.0;
        for i in 0..1000 {
            (self.fun)(i, &mut out);
        }
        out
    }

    fn match_check_call(&mut self) -> f64 {
        let mut out = 0.0;
        for i in 0..1000 {
            match self.mode {
                TestMode::A => { self.a(i, &mut out); }
                TestMode::B => { self.b(i, &mut out); }
                TestMode::C => { self.c(i, &mut out); }
                TestMode::D => { self.d(i, &mut out); }
                TestMode::E => { self.e(i, &mut out); }
            }
        }
        out
    }

    fn match_check_inner(&mut self) -> f64 {
        let mut out = 0.0;
            match self.mode {
                TestMode::A => {
        for i in 0..1000 {
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
        }
                }
                TestMode::B => {
        for i in 0..1000 {
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
        }
                }
                TestMode::C => {
        for i in 0..1000 {
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
        }
                }
                TestMode::D => {
        for i in 0..1000 {
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                    out -= ((i as f64) * std::f64::consts::PI / 1000.0).sin();
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
        }
                }
                TestMode::E => {
        for i in 0..1000 {
                    out -= ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
                    out -= ((i as f64) * std::f64::consts::PI / 1000.0).cos();
        }
                }
            }
        out
   }

    fn match_check(&mut self) -> f64 {
        let mut out = 0.0;
        for i in 0..1000 {
            match self.mode {
                TestMode::A => {
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
                }
                TestMode::B => {
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                }
                TestMode::C => {
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
                }
                TestMode::D => {
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                    out -= ((i as f64) * std::f64::consts::PI / 1000.0).sin();
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                }
                TestMode::E => {
                    out -= ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                    out += ((i as f64) * std::f64::consts::PI / 1000.0).sin();
                    out -= ((i as f64) * std::f64::consts::PI / 1000.0).cos();
                }
            }
        }
        out
   }
}

fn main() {


    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for i in 0..100000000 {
        res += (((i % 1000) as f64 * std::f64::consts::PI) / 1000.0).sin().fract().abs();
    }
    println!("fract Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for i in 0..100000000 {
        res += ((((i % 1000) as f64 * std::f64::consts::PI) / 1000.0).sin() + 1.0) * 0.5;
    }
    println!("plusmul Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for i in 0..100000000 {
        res += ((((i % 1000) as f64 * std::f64::consts::PI) / 1000.0).sin() + 1.0) / 2.0;
    }
    println!("plusdiv Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for i in 0..100000000 {
        let sig = (((i % 1000) as f64 * std::f64::consts::PI) / 1000.0).sin();
        if i % 4 == 0 {
            res += sig;
        } else {
            res += (sig + 1.0) * 0.5;
        }
    }
    println!("branch Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let mut t1 = Test { mode: TestMode::A, fun: Box::new(|i: i64, out: &mut f64| *out = i as f64), ft: Box::new(A) };
    let mut t2 = Test { mode: TestMode::B, fun: Box::new(|i: i64, out: &mut f64| *out = i as f64), ft: Box::new(A) };
    let mut t3 = Test { mode: TestMode::C, fun: Box::new(|i: i64, out: &mut f64| *out = i as f64), ft: Box::new(A) };
    let mut t4 = Test { mode: TestMode::D, fun: Box::new(|i: i64, out: &mut f64| *out = i as f64), ft: Box::new(A) };
    let mut t5 = Test { mode: TestMode::E, fun: Box::new(|i: i64, out: &mut f64| *out = i as f64), ft: Box::new(A) };

    t1.init_match_check();
    t2.init_match_check();
    t3.init_match_check();
    t4.init_match_check();
    t5.init_match_check();

    use kickmessvst::helpers::*;
    init_cos_tab();

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for i in 0..100000000 {
        res += (((i % 1000) as f64 * std::f64::consts::PI) / 1000.0).sin();
    }
    println!("Sin Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res2 = 0.0;
    for i in 0..100000000 {
        res2 += fast_sin(((i % 1000) as f64 * std::f64::consts::PI) / 1000.0);
    }
    println!("FastSin Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res2);
    println!("Error: {}", (res - res2).abs());

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for _x in 0..10000 {
        res += t1.match_check();
        res += t2.match_check();
        res += t3.match_check();
        res += t4.match_check();
        res += t5.match_check();
    }
    println!("Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for _x in 0..10000 {
        res += t1.match_check_call();
        res += t2.match_check_call();
        res += t3.match_check_call();
        res += t4.match_check_call();
        res += t5.match_check_call();
    }
    println!("Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for _x in 0..10000 {
        res += t1.match_check_inner();
        res += t2.match_check_inner();
        res += t3.match_check_inner();
        res += t4.match_check_inner();
        res += t5.match_check_inner();
    }
    println!("Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for _x in 0..10000 {
        res += t1.match_check_direct();
        res += t2.match_check_direct();
        res += t3.match_check_direct();
        res += t4.match_check_direct();
        res += t5.match_check_direct();
    }
    println!("Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for _x in 0..10000 {
        res += t1.match_check_vt();
        res += t2.match_check_vt();
        res += t3.match_check_vt();
        res += t4.match_check_vt();
        res += t5.match_check_vt();
    }
    println!("Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);

    let ta = std::time::Instant::now();
    let mut res = 0.0;
    for _x in 0..10000 {
        let mut out = 0.0;
        for i in 0..1000 { t1.a(i, &mut out); }
        res += out;
        let mut out = 0.0;
        for i in 0..1000 { t1.b(i, &mut out); }
        res += out;
        let mut out = 0.0;
        for i in 0..1000 { t1.c(i, &mut out); }
        res += out;
        let mut out = 0.0;
        for i in 0..1000 { t1.d(i, &mut out); }
        res += out;
        let mut out = 0.0;
        for i in 0..1000 { t1.e(i, &mut out); }
        res += out;
    }
    println!("Elapsed: {:?} ({})", std::time::Instant::now().duration_since(ta), res);
}
