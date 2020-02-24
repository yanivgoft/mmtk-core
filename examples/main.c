#include <stdio.h>
#include "mmtk.h"

static bool mmtk_is_mutator(void* tls) {
    return true;
}

OpenJDK_Upcalls mmtk_upcalls = {
    NULL, // stop_all_mutators
    NULL, // resume_mutators
    NULL, // spawn_collector_thread
    NULL, // block_for_gc
    NULL, // active_collector
    NULL, // get_next_mutator
    NULL, // reset_mutator_iterator
    NULL, // compute_static_roots
    NULL, // compute_global_roots
    NULL, // compute_thread_roots
    NULL, // scan_object
    NULL, // dump_object
    NULL, // get_object_size
    NULL, // get_mmtk_mutator
    mmtk_is_mutator,
};

int main(int argc, char* argv[]){
    openjdk_gc_init(&mmtk_upcalls, 1024*1024);

    MMTk_Mutator handle = bind_mutator(0);
    
    for (int i=0;i<4;i++){
        int arr_size = 10000;
        int* my_arr = alloc(handle, sizeof(int)*arr_size, 8, 0, 0);
        if (!my_arr){
            printf("OOM\n");
            break;
        }
        for (int j=0;j<arr_size;j++){
            my_arr[j]=j;
        }
        for (int j=0;j<arr_size;j++){
            if (my_arr[j]!=j){
                printf("Sanity check failed\n");
            }
        }
        printf("%p\n", my_arr);
    }
    return 0;
}
