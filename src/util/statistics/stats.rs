use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::sync::Mutex;
use util::statistics::Timer;
use util::statistics::counter::{Counter, LongCounter};
use util::statistics::counter::MonotoneNanoTime;

lazy_static! {
    pub static ref STATS: Stats = Stats::new();
    pub static ref COUNTER: Mutex<Vec<Box<Counter + Send>>> = Mutex::new(Vec::new());
}

// FIXME overflow detection
//static PHASE: AtomicUsize = AtomicUsize::new(0);
//static GATHERING_STATS: AtomicBool = AtomicBool::new(false);
//static EXCEEDED_PHASE_LIMIT: AtomicBool = AtomicBool::new(false);

pub const MAX_PHASES: usize = 1 << 12;
pub const MAX_COUNTERS: usize = 100;

pub fn new_counter<T: Counter + Send + 'static>(c: T) -> usize {
    let mut counter = COUNTER.lock().unwrap();
    counter.push(Box::new(c));
    return counter.len() - 1;
}

pub struct Stats {
    gc_count: AtomicUsize,
    total_time: usize,

    phase: AtomicUsize,
    gathering_stats: AtomicBool,
    exceeded_phase_limit: AtomicBool,
}

impl Stats {
    pub fn new() -> Self {
        let t: Timer = LongCounter::new("time".to_string(), true, false);
        Stats {
            gc_count: AtomicUsize::new(0),
            total_time: new_counter(t),

            phase: AtomicUsize::new(0),
            gathering_stats: AtomicBool::new(false),
            exceeded_phase_limit: AtomicBool::new(false),
        }
    }

    pub fn start_gc(&self) {
        let mut counter = COUNTER.lock().unwrap();
        self.gc_count.fetch_add(1, Ordering::SeqCst);
        if !self.get_gathering_stats() {
            return;
        }
        if self.get_phase() < MAX_PHASES - 1 {
            counter[self.total_time].phase_change(self.get_phase());
            self.increment_phase();
        } else {
            if !self.exceeded_phase_limit.load(Ordering::SeqCst) {
                println!("Warning: number of GC phases exceeds MAX_PHASES");
                self.exceeded_phase_limit.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn end_gc(&self) {
        let mut counter = COUNTER.lock().unwrap();
        if !self.get_gathering_stats() {
            return;
        }
        if self.get_phase() < MAX_PHASES - 1 {
            counter[self.total_time].phase_change(self.get_phase());
            self.increment_phase();
        } else {
            if !self.exceeded_phase_limit.load(Ordering::SeqCst) {
                println!("Warning: number of GC phases exceeds MAX_PHASES");
                self.exceeded_phase_limit.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn print_stats(&self) {
        println!("============================ MMTk Statistics Totals ============================");
        self.print_column_names();
        print!("{}\t", self.get_phase() / 2);
        let counter = COUNTER.lock().unwrap();
        for c in &(*counter) {
            if c.merge_phases() {
                c.print_total(None);
                print!("\t");
            } else {
                c.print_total(Some(true));
                print!("\t");
                c.print_total(Some(false));
                print!("\t");
            }
        }
        println!();
        print!("Total time: ");
        counter[self.total_time].print_total(None);
        println!(" ms");
        println!("------------------------------ End MMTk Statistics -----------------------------")
    }

    pub fn print_column_names(&self) {
        print!("GC\t");
        let counter = COUNTER.lock().unwrap();
        for c in &(*counter) {
            if c.merge_phases() {
                print!("{}\t", c.name());
            } else {
                print!("{}.mu\t{}.gc\t", c.name(), c.name());
            }
        }
        print!("\n");
    }

    pub fn start_all(&self) {
        let mut counter = COUNTER.lock().unwrap();
        if self.get_gathering_stats() {
            println!("Error: calling Stats.startAll() while stats running");
            println!("       verbosity > 0 and the harness mechanism may be conflicting");
            debug_assert!(false);
        }
        self.set_gathering_stats(true);
        if counter[self.total_time].implicitly_start() {
            counter[self.total_time].start()
        }
    }

    pub fn stop_all(&self) {
        self.stop_all_counters();
        self.print_stats();
    }

    fn stop_all_counters(&self) {
        let mut counter = COUNTER.lock().unwrap();
        counter[self.total_time].stop();
        self.set_gathering_stats(false);
    }

    fn increment_phase(&self) {
        self.phase.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_phase(&self) -> usize {
        self.phase.load(Ordering::SeqCst)
    }

    pub fn get_gathering_stats(&self) -> bool {
        self.gathering_stats.load(Ordering::SeqCst)
    }

    fn set_gathering_stats(&self, val: bool) {
        self.gathering_stats.store(val, Ordering::SeqCst);
    }
}
