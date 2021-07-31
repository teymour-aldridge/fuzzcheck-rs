extern int __llvm_profile_runtime = 0;

extern unsigned long int
    CountersStart __asm("section$start$__DATA$__llvm_prf_cnts"); // different based on the platform, so double-check it
extern unsigned long int
    CountersEnd __asm("section$end$__DATA$__llvm_prf_cnts");

unsigned long int * get_start_instrumentation_counters() {
    return &CountersStart;
}
unsigned long int * get_end_instrumentation_counters() {
    return &CountersEnd;
}
