(function() {var implementors = {};
implementors["mmtk"] = [{"text":"impl&lt;P&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/struct.Mutator.html\" title=\"struct mmtk::Mutator\">Mutator</a>&lt;P&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;P as <a class=\"trait\" href=\"mmtk/trait.Plan.html\" title=\"trait mmtk::Plan\">Plan</a>&gt;::<a class=\"type\" href=\"mmtk/trait.Plan.html#associatedtype.VM\" title=\"type mmtk::Plan::VM\">VM</a>: <a class=\"trait\" href=\"mmtk/vm/trait.VMBinding.html\" title=\"trait mmtk::vm::VMBinding\">VMBinding</a>,&nbsp;</span>","synthetic":true,"types":["mmtk::plan::mutator_context::Mutator"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"enum\" href=\"mmtk/enum.AllocationSemantics.html\" title=\"enum mmtk::AllocationSemantics\">AllocationSemantics</a>","synthetic":true,"types":["mmtk::plan::global::AllocationSemantics"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/struct.SynchronizedCounter.html\" title=\"struct mmtk::util::SynchronizedCounter\">SynchronizedCounter</a>","synthetic":true,"types":["mmtk::util::synchronized_counter::SynchronizedCounter"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/address/struct.Address.html\" title=\"struct mmtk::util::address::Address\">Address</a>","synthetic":true,"types":["mmtk::util::address::Address"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/address/struct.ObjectReference.html\" title=\"struct mmtk::util::address::ObjectReference\">ObjectReference</a>","synthetic":true,"types":["mmtk::util::address::ObjectReference"]},{"text":"impl&lt;VM&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/alloc/struct.BumpAllocator.html\" title=\"struct mmtk::util::alloc::BumpAllocator\">BumpAllocator</a>&lt;VM&gt;","synthetic":true,"types":["mmtk::util::alloc::bumpallocator::BumpAllocator"]},{"text":"impl&lt;VM&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/alloc/allocators/struct.Allocators.html\" title=\"struct mmtk::util::alloc::allocators::Allocators\">Allocators</a>&lt;VM&gt;","synthetic":true,"types":["mmtk::util::alloc::allocators::Allocators"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"enum\" href=\"mmtk/util/alloc/allocators/enum.AllocatorSelector.html\" title=\"enum mmtk::util::alloc::allocators::AllocatorSelector\">AllocatorSelector</a>","synthetic":true,"types":["mmtk::util::alloc::allocators::AllocatorSelector"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/alloc/dump_linear_scan/struct.DumpLinearScan.html\" title=\"struct mmtk::util::alloc::dump_linear_scan::DumpLinearScan\">DumpLinearScan</a>","synthetic":true,"types":["mmtk::util::alloc::dump_linear_scan::DumpLinearScan"]},{"text":"impl&lt;VM&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/alloc/large_object_allocator/struct.LargeObjectAllocator.html\" title=\"struct mmtk::util::alloc::large_object_allocator::LargeObjectAllocator\">LargeObjectAllocator</a>&lt;VM&gt;","synthetic":true,"types":["mmtk::util::alloc::large_object_allocator::LargeObjectAllocator"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/heap/struct.HeapMeta.html\" title=\"struct mmtk::util::heap::HeapMeta\">HeapMeta</a>","synthetic":true,"types":["mmtk::util::heap::heap_meta::HeapMeta"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"enum\" href=\"mmtk/util/heap/enum.VMRequest.html\" title=\"enum mmtk::util::heap::VMRequest\">VMRequest</a>","synthetic":true,"types":["mmtk::util::heap::vmrequest::VMRequest"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/heap/layout/struct.FragmentedMapper.html\" title=\"struct mmtk::util::heap::layout::FragmentedMapper\">FragmentedMapper</a>","synthetic":true,"types":["mmtk::util::heap::layout::fragmented_mapper::FragmentedMapper"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/heap/layout/map64/struct.Map64.html\" title=\"struct mmtk::util::heap::layout::map64::Map64\">Map64</a>","synthetic":true,"types":["mmtk::util::heap::layout::map64::Map64"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/heap/freelistpageresource/struct.CommonFreeListPageResource.html\" title=\"struct mmtk::util::heap::freelistpageresource::CommonFreeListPageResource\">CommonFreeListPageResource</a>","synthetic":true,"types":["mmtk::util::heap::freelistpageresource::CommonFreeListPageResource"]},{"text":"impl&lt;VM&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/heap/freelistpageresource/struct.FreeListPageResource.html\" title=\"struct mmtk::util::heap::freelistpageresource::FreeListPageResource\">FreeListPageResource</a>&lt;VM&gt;","synthetic":true,"types":["mmtk::util::heap::freelistpageresource::FreeListPageResource"]},{"text":"impl&lt;VM&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/heap/monotonepageresource/struct.MonotonePageResource.html\" title=\"struct mmtk::util::heap::monotonepageresource::MonotonePageResource\">MonotonePageResource</a>&lt;VM&gt;","synthetic":true,"types":["mmtk::util::heap::monotonepageresource::MonotonePageResource"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"enum\" href=\"mmtk/util/heap/monotonepageresource/enum.MonotonePageResourceConditional.html\" title=\"enum mmtk::util::heap::monotonepageresource::MonotonePageResourceConditional\">MonotonePageResourceConditional</a>","synthetic":true,"types":["mmtk::util::heap::monotonepageresource::MonotonePageResourceConditional"]},{"text":"impl&lt;VM&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/heap/pageresource/struct.CommonPageResource.html\" title=\"struct mmtk::util::heap::pageresource::CommonPageResource\">CommonPageResource</a>&lt;VM&gt;","synthetic":true,"types":["mmtk::util::heap::pageresource::CommonPageResource"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/heap/space_descriptor/struct.SpaceDescriptor.html\" title=\"struct mmtk::util::heap::space_descriptor::SpaceDescriptor\">SpaceDescriptor</a>","synthetic":true,"types":["mmtk::util::heap::space_descriptor::SpaceDescriptor"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/int_array_freelist/struct.IntArrayFreeList.html\" title=\"struct mmtk::util::int_array_freelist::IntArrayFreeList\">IntArrayFreeList</a>","synthetic":true,"types":["mmtk::util::int_array_freelist::IntArrayFreeList"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/options/struct.UnsafeOptionsWrapper.html\" title=\"struct mmtk::util::options::UnsafeOptionsWrapper\">UnsafeOptionsWrapper</a>","synthetic":true,"types":["mmtk::util::options::UnsafeOptionsWrapper"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/options/struct.Options.html\" title=\"struct mmtk::util::options::Options\">Options</a>","synthetic":true,"types":["mmtk::util::options::Options"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"enum\" href=\"mmtk/util/options/enum.NurseryZeroingOptions.html\" title=\"enum mmtk::util::options::NurseryZeroingOptions\">NurseryZeroingOptions</a>","synthetic":true,"types":["mmtk::util::options::NurseryZeroingOptions"]},{"text":"impl&lt;'a, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/queue/struct.LocalQueue.html\" title=\"struct mmtk::util::queue::LocalQueue\">LocalQueue</a>&lt;'a, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,&nbsp;</span>","synthetic":true,"types":["mmtk::util::queue::local_queue::LocalQueue"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/queue/struct.SharedQueue.html\" title=\"struct mmtk::util::queue::SharedQueue\">SharedQueue</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,&nbsp;</span>","synthetic":true,"types":["mmtk::util::queue::shared_queue::SharedQueue"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/raw_memory_freelist/struct.RawMemoryFreeList.html\" title=\"struct mmtk::util::raw_memory_freelist::RawMemoryFreeList\">RawMemoryFreeList</a>","synthetic":true,"types":["mmtk::util::raw_memory_freelist::RawMemoryFreeList"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/reference_processor/struct.ReferenceProcessors.html\" title=\"struct mmtk::util::reference_processor::ReferenceProcessors\">ReferenceProcessors</a>","synthetic":true,"types":["mmtk::util::reference_processor::ReferenceProcessors"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/reference_processor/struct.ReferenceProcessor.html\" title=\"struct mmtk::util::reference_processor::ReferenceProcessor\">ReferenceProcessor</a>","synthetic":true,"types":["mmtk::util::reference_processor::ReferenceProcessor"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"enum\" href=\"mmtk/util/reference_processor/enum.Semantics.html\" title=\"enum mmtk::util::reference_processor::Semantics\">Semantics</a>","synthetic":true,"types":["mmtk::util::reference_processor::Semantics"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/statistics/counter/struct.MonotoneNanoTime.html\" title=\"struct mmtk::util::statistics::counter::MonotoneNanoTime\">MonotoneNanoTime</a>","synthetic":true,"types":["mmtk::util::statistics::counter::MonotoneNanoTime"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/statistics/counter/struct.LongCounter.html\" title=\"struct mmtk::util::statistics::counter::LongCounter\">LongCounter</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;T as <a class=\"trait\" href=\"mmtk/util/statistics/counter/trait.Diffable.html\" title=\"trait mmtk::util::statistics::counter::Diffable\">Diffable</a>&gt;::<a class=\"type\" href=\"mmtk/util/statistics/counter/trait.Diffable.html#associatedtype.Val\" title=\"type mmtk::util::statistics::counter::Diffable::Val\">Val</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,&nbsp;</span>","synthetic":true,"types":["mmtk::util::statistics::counter::LongCounter"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/statistics/stats/struct.SharedStats.html\" title=\"struct mmtk::util::statistics::stats::SharedStats\">SharedStats</a>","synthetic":true,"types":["mmtk::util::statistics::stats::SharedStats"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/statistics/stats/struct.Stats.html\" title=\"struct mmtk::util::statistics::stats::Stats\">Stats</a>","synthetic":true,"types":["mmtk::util::statistics::stats::Stats"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/treadmill/struct.TreadMill.html\" title=\"struct mmtk::util::treadmill::TreadMill\">TreadMill</a>","synthetic":true,"types":["mmtk::util::treadmill::TreadMill"]},{"text":"impl !<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/policy/space/struct.SFTMap.html\" title=\"struct mmtk::policy::space::SFTMap\">SFTMap</a>","synthetic":true,"types":["mmtk::policy::space::SFTMap"]},{"text":"impl&lt;VM&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/policy/space/struct.CommonSpace.html\" title=\"struct mmtk::policy::space::CommonSpace\">CommonSpace</a>&lt;VM&gt;","synthetic":true,"types":["mmtk::policy::space::CommonSpace"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/policy/space/struct.SpaceOptions.html\" title=\"struct mmtk::policy::space::SpaceOptions\">SpaceOptions</a>","synthetic":true,"types":["mmtk::policy::space::SpaceOptions"]},{"text":"impl&lt;VM&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/policy/immortalspace/struct.ImmortalSpace.html\" title=\"struct mmtk::policy::immortalspace::ImmortalSpace\">ImmortalSpace</a>&lt;VM&gt;","synthetic":true,"types":["mmtk::policy::immortalspace::ImmortalSpace"]},{"text":"impl&lt;VM&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/policy/copyspace/struct.CopySpace.html\" title=\"struct mmtk::policy::copyspace::CopySpace\">CopySpace</a>&lt;VM&gt;","synthetic":true,"types":["mmtk::policy::copyspace::CopySpace"]},{"text":"impl&lt;VM&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/policy/largeobjectspace/struct.LargeObjectSpace.html\" title=\"struct mmtk::policy::largeobjectspace::LargeObjectSpace\">LargeObjectSpace</a>&lt;VM&gt;","synthetic":true,"types":["mmtk::policy::largeobjectspace::LargeObjectSpace"]},{"text":"impl&lt;C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/struct.WorkerGroup.html\" title=\"struct mmtk::scheduler::WorkerGroup\">WorkerGroup</a>&lt;C&gt;","synthetic":true,"types":["mmtk::scheduler::worker::WorkerGroup"]},{"text":"impl&lt;C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"enum\" href=\"mmtk/scheduler/enum.CoordinatorMessage.html\" title=\"enum mmtk::scheduler::CoordinatorMessage\">CoordinatorMessage</a>&lt;C&gt;","synthetic":true,"types":["mmtk::scheduler::scheduler::CoordinatorMessage"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/stat/struct.SchedulerStat.html\" title=\"struct mmtk::scheduler::stat::SchedulerStat\">SchedulerStat</a>","synthetic":true,"types":["mmtk::scheduler::stat::SchedulerStat"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/stat/struct.WorkStat.html\" title=\"struct mmtk::scheduler::stat::WorkStat\">WorkStat</a>","synthetic":true,"types":["mmtk::scheduler::stat::WorkStat"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/stat/struct.WorkerLocalStat.html\" title=\"struct mmtk::scheduler::stat::WorkerLocalStat\">WorkerLocalStat</a>","synthetic":true,"types":["mmtk::scheduler::stat::WorkerLocalStat"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.ScheduleCollection.html\" title=\"struct mmtk::scheduler::gc_works::ScheduleCollection\">ScheduleCollection</a>","synthetic":true,"types":["mmtk::scheduler::gc_works::ScheduleCollection"]},{"text":"impl&lt;P&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.Prepare.html\" title=\"struct mmtk::scheduler::gc_works::Prepare\">Prepare</a>&lt;P&gt;","synthetic":true,"types":["mmtk::scheduler::gc_works::Prepare"]},{"text":"impl&lt;VM&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.PrepareMutator.html\" title=\"struct mmtk::scheduler::gc_works::PrepareMutator\">PrepareMutator</a>&lt;VM&gt;","synthetic":true,"types":["mmtk::scheduler::gc_works::PrepareMutator"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.PrepareCollector.html\" title=\"struct mmtk::scheduler::gc_works::PrepareCollector\">PrepareCollector</a>","synthetic":true,"types":["mmtk::scheduler::gc_works::PrepareCollector"]},{"text":"impl&lt;P&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.Release.html\" title=\"struct mmtk::scheduler::gc_works::Release\">Release</a>&lt;P&gt;","synthetic":true,"types":["mmtk::scheduler::gc_works::Release"]},{"text":"impl&lt;VM&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.ReleaseMutator.html\" title=\"struct mmtk::scheduler::gc_works::ReleaseMutator\">ReleaseMutator</a>&lt;VM&gt;","synthetic":true,"types":["mmtk::scheduler::gc_works::ReleaseMutator"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.ReleaseCollector.html\" title=\"struct mmtk::scheduler::gc_works::ReleaseCollector\">ReleaseCollector</a>","synthetic":true,"types":["mmtk::scheduler::gc_works::ReleaseCollector"]},{"text":"impl&lt;ScanEdges&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.StopMutators.html\" title=\"struct mmtk::scheduler::gc_works::StopMutators\">StopMutators</a>&lt;ScanEdges&gt;","synthetic":true,"types":["mmtk::scheduler::gc_works::StopMutators"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.EndOfGC.html\" title=\"struct mmtk::scheduler::gc_works::EndOfGC\">EndOfGC</a>","synthetic":true,"types":["mmtk::scheduler::gc_works::EndOfGC"]},{"text":"impl&lt;Edges&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.ScanStackRoots.html\" title=\"struct mmtk::scheduler::gc_works::ScanStackRoots\">ScanStackRoots</a>&lt;Edges&gt;","synthetic":true,"types":["mmtk::scheduler::gc_works::ScanStackRoots"]},{"text":"impl&lt;Edges&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.ScanStackRoot.html\" title=\"struct mmtk::scheduler::gc_works::ScanStackRoot\">ScanStackRoot</a>&lt;Edges&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;Edges as <a class=\"trait\" href=\"mmtk/scheduler/gc_works/trait.ProcessEdgesWork.html\" title=\"trait mmtk::scheduler::gc_works::ProcessEdgesWork\">ProcessEdgesWork</a>&gt;::<a class=\"type\" href=\"mmtk/scheduler/gc_works/trait.ProcessEdgesWork.html#associatedtype.VM\" title=\"type mmtk::scheduler::gc_works::ProcessEdgesWork::VM\">VM</a>: <a class=\"trait\" href=\"mmtk/vm/trait.VMBinding.html\" title=\"trait mmtk::vm::VMBinding\">VMBinding</a>,&nbsp;</span>","synthetic":true,"types":["mmtk::scheduler::gc_works::ScanStackRoot"]},{"text":"impl&lt;Edges&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.ScanVMSpecificRoots.html\" title=\"struct mmtk::scheduler::gc_works::ScanVMSpecificRoots\">ScanVMSpecificRoots</a>&lt;Edges&gt;","synthetic":true,"types":["mmtk::scheduler::gc_works::ScanVMSpecificRoots"]},{"text":"impl&lt;Edges&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.ScanObjects.html\" title=\"struct mmtk::scheduler::gc_works::ScanObjects\">ScanObjects</a>&lt;Edges&gt;","synthetic":true,"types":["mmtk::scheduler::gc_works::ScanObjects"]},{"text":"impl&lt;E&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.ProcessModBuf.html\" title=\"struct mmtk::scheduler::gc_works::ProcessModBuf\">ProcessModBuf</a>&lt;E&gt;","synthetic":true,"types":["mmtk::scheduler::gc_works::ProcessModBuf"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/util/opaque_pointer/struct.OpaquePointer.html\" title=\"struct mmtk::util::opaque_pointer::OpaquePointer\">OpaquePointer</a>","synthetic":false,"types":["mmtk::util::opaque_pointer::OpaquePointer"]},{"text":"impl&lt;VM:&nbsp;<a class=\"trait\" href=\"mmtk/vm/trait.VMBinding.html\" title=\"trait mmtk::vm::VMBinding\">VMBinding</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/struct.MMTK.html\" title=\"struct mmtk::MMTK\">MMTK</a>&lt;VM&gt;","synthetic":false,"types":["mmtk::mmtk::MMTK"]},{"text":"impl&lt;C:&nbsp;<a class=\"trait\" href=\"mmtk/scheduler/trait.Context.html\" title=\"trait mmtk::scheduler::Context\">Context</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/struct.Scheduler.html\" title=\"struct mmtk::scheduler::Scheduler\">Scheduler</a>&lt;C&gt;","synthetic":false,"types":["mmtk::scheduler::scheduler::Scheduler"]},{"text":"impl&lt;C:&nbsp;<a class=\"trait\" href=\"mmtk/scheduler/trait.Context.html\" title=\"trait mmtk::scheduler::Context\">Context</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/struct.Worker.html\" title=\"struct mmtk::scheduler::Worker\">Worker</a>&lt;C&gt;","synthetic":false,"types":["mmtk::scheduler::worker::Worker"]},{"text":"impl&lt;E:&nbsp;<a class=\"trait\" href=\"mmtk/scheduler/gc_works/trait.ProcessEdgesWork.html\" title=\"trait mmtk::scheduler::gc_works::ProcessEdgesWork\">ProcessEdgesWork</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"mmtk/scheduler/gc_works/struct.ProcessEdgesBase.html\" title=\"struct mmtk::scheduler::gc_works::ProcessEdgesBase\">ProcessEdgesBase</a>&lt;E&gt;","synthetic":false,"types":["mmtk::scheduler::gc_works::ProcessEdgesBase"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()