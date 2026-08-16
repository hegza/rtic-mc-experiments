pub mod app
{
    #[doc = r" Include peripheral crate(s) that defines the vector table"] use
    bsp as _; use Obs as __rtic_obs; #[repr(u8)]
    #[doc =
    r" Discriminates the tasks of this application for observability hooks."]
    pub enum TaskId { SomeTask, Sw1, Core0Priority2Dispatcher, }
    #[allow(non_camel_case_types)] #[repr(u8)]
    #[doc =
    r" Discriminates the shared resources of this application for observability hooks."]
    pub enum ResourceId { Uart, } use bsp ::
    {
        CPU_FREQ_HZ, apb_uart :: ApbUart, mmap :: apb_timer :: TIMER0_ADDR,
        sprintln, timer_group :: Timer,
    }; use fugit :: ExtU32; #[doc = r" Module defining rtic traits"] pub use
    rtic_traits :: * ; pub mod rtic_traits
    {
        #[doc = r" Trait for a hardware task"] pub trait RticTask
        {
            #[doc =
            r" Associated type that can be used to make [Self::init] take arguments"]
            type InitArgs : Sized;
            #[doc = r" Task local variables initialization routine"] fn
            init(args : Self :: InitArgs) -> Self;
            #[doc = r" Function to be bound to a HW Interrupt"] fn
            exec(& mut self);
        } #[doc = r" Trait for an idle task"] pub trait RticIdleTask
        {
            #[doc =
            r" Associated type that can be used to make [Self::init] take arguments"]
            type InitArgs : Sized;
            #[doc = r" Task local variables initialization routine"] fn
            init(args : Self :: InitArgs) -> Self;
            #[doc =
            r" Function to be executing when no other task is running"] fn
            exec(& mut self) -> ! ;
        } pub trait RticMutex
        {
            type ResourceType; fn lock < R >
            (& mut self, f : impl FnOnce(& mut Self :: ResourceType) -> R) ->
            R;
        } pub trait RticReadLock
        {
            type ResourceType; fn read_lock < R >
            (& self, f : impl FnOnce(& Self :: ResourceType) -> R) -> R;
        } use super :: { TaskId, ResourceId };
        #[doc = r" Optional, zero-cost-when-disabled observability hooks."]
        #[doc = r""]
        #[doc =
        r" A type referenced by the `#[app(obs = ...)]` argument may implement"]
        #[doc = r" this trait to observe RTIC scheduling dynamics:"]
        #[doc = r""]
        #[doc =
        r" - `on_task_act` (task *activation*): a task's job started executing."]
        #[doc =
        r" - `on_task_comp` (task *completion*): a task's job finished executing."]
        #[doc =
        r" - `on_res_acq` (resource *acquire*, i.e. lock): entering the SRP"]
        #[doc =
        r"   critical section, before the interrupt ceiling is raised."]
        #[doc =
        r" - `on_res_rel` (resource *release*, i.e. unlock): leaving the SRP"]
        #[doc =
        r"   critical section, after the interrupt ceiling is restored."]
        #[doc = r""]
        #[doc =
        r" All methods are static and have default no-op bodies, so a user may"]
        #[doc =
        r" implement only the hooks they care about. When `obs` is not"]
        #[doc = r" provided, no hook calls are generated at all."] pub trait
        RticObservability
        {
            #[doc =
            r" Task activation: the task's job has started executing."]
            #[inline(always)] fn on_task_act(task : TaskId) {}
            #[doc =
            r" Task completion: the task's job has finished executing."]
            #[inline(always)] fn on_task_comp(task : TaskId) {}
            #[doc =
            r" Resource acquire (lock): entering the SRP critical section,"]
            #[doc = r" before the interrupt ceiling is raised."]
            #[inline(always)] fn
            on_res_acq(res : ResourceId, task_prio : u16, ceiling : u16) {}
            #[doc =
            r" Resource release (unlock): leaving the SRP critical section,"]
            #[doc = r" after the interrupt ceiling is restored."]
            #[inline(always)] fn
            on_res_rel(res : ResourceId, task_prio : u16, ceiling : u16) {}
        }
    } #[doc = r" critical section function"] #[inline] pub fn
    __rtic_interrupt_free < F, R > (f : F) -> R where F : FnOnce() -> R,
    {
        rtic :: export :: interrupt_disable(); let r = f(); unsafe
        { rtic :: export :: interrupt_enable(); } r
    } struct Obs; impl RticObservability for Obs {} static mut
    __rtic_internal__Sw1__INPUTS : rtic :: export :: Queue < < Sw1 as
    RticSwTask > :: SpawnInput, 2 > = rtic :: export :: Queue :: new(); impl
    Sw1
    {
        pub fn spawn(input : < Sw1 as RticSwTask > :: SpawnInput) -> Result <
        (), < Sw1 as RticSwTask > :: SpawnInput >
        {
            let mut inputs_producer = unsafe
            { __rtic_internal__Sw1__INPUTS.split().0 }; let mut ready_producer
            = unsafe { __rtic_internal__Core0Prio2Tasks__RQ.split().0 };
            #[doc =
            r" need to protect by a critical section because many producers of different priorities can spawn/enqueue this task"]
            __rtic_interrupt_free(| | -> Result < (), < Sw1 as RticSwTask > ::
            SpawnInput >
            {
                inputs_producer.enqueue(input) ? ; unsafe
                { ready_producer.enqueue_unchecked(Core0Prio2Tasks :: Sw1) };
                __rtic_local_irq_pend(rtic :: export :: interrupts ::
                MachineSoft); Ok(())
            })
        }
    } #[doc = " Dispatchers of"] #[doc = " Core 0"] #[derive(Clone, Copy)]
    #[doc(hidden)] pub enum Core0Prio2Tasks { Sw1, } #[doc(hidden)]
    #[allow(non_upper_case_globals)] static mut
    __rtic_internal__Core0Prio2Tasks__RQ : rtic :: export :: Queue <
    Core0Prio2Tasks, 2usize > = rtic :: export :: Queue :: new();
    #[doc = r" RTIC Software task trait"]
    #[doc = r" Trait for a software task"] pub trait RticSwTask
    {
        type InitArgs : Sized; type SpawnInput;
        #[doc = r" Task local variables initialization routine"] fn
        init(args : Self :: InitArgs) -> Self;
        #[doc =
        r" Function to be executing when the scheduled software task is dispatched"]
        fn exec(& mut self, input : Self :: SpawnInput);
    } #[doc = r" Core local interrupt pending"] #[doc(hidden)] #[inline] pub
    fn __rtic_local_irq_pend < I : rtic :: export :: InterruptNumber >
    (irq_nbr : I) { rtic :: export :: pend(irq_nbr); } #[doc = " # CORE 0"]
    static mut SHARED : core :: mem :: MaybeUninit < Shared > = core :: mem ::
    MaybeUninit :: uninit(); struct Shared { uart : ApbUart, } fn init() ->
    Shared
    {
        let uart = ApbUart :: init(CPU_FREQ_HZ, 115_200); let mut timer =
        Timer :: init :: < TIMER0_ADDR > ().into_periodic(); sprintln!
        ("init"); timer.set_period(10_u32.micros()); timer.start(); Shared
        { uart }
    } static mut SOME_TASK : core :: mem :: MaybeUninit < SomeTask > = core ::
    mem :: MaybeUninit :: uninit(); struct SomeTask; const _ : fn() = ||
    { __rtic_trait_checks :: implements_rtic_task :: < SomeTask > (); }; impl
    RticTask for SomeTask
    {
        fn init(_ : ()) -> Self { Self } fn exec(& mut self)
        {
            < __rtic_obs as RticObservability > ::
            on_task_act(TaskId :: SomeTask);
            {
                self.shared().uart.lock(| _uart |
                { sprintln! ("T"); sprintln! ("1"); }); Sw1 ::
                spawn(()).unwrap();
                self.shared().uart.lock(| _uart |
                { sprintln! ("T"); sprintln! ("2"); });
            } < __rtic_obs as RticObservability > ::
            on_task_comp(TaskId :: SomeTask);
        } type InitArgs = ();
    } impl SomeTask { pub const fn priority() -> u16 { 1u16 } } impl SomeTask
    {
        pub fn shared(& self) -> __some_task_shared_resources
        {
            const TASK_PRIORITY : u16 = 1u16; __some_task_shared_resources ::
            new(TASK_PRIORITY)
        }
    } pub struct __some_task_shared_resources { pub uart : __uart_mutex, }
    impl __some_task_shared_resources
    {
        #[inline(always)] pub fn new(priority : u16) -> Self
        { Self { uart : __uart_mutex :: new(priority), } }
    } impl SomeTask
    {
        pub const fn current_core() -> __rtic__internal__Core0
        { unsafe { __rtic__internal__Core0 :: new() } }
    } static mut SW1 : core :: mem :: MaybeUninit < Sw1 > = core :: mem ::
    MaybeUninit :: uninit(); #[doc = " Software tasks of"] #[doc = " Core 0"]
    struct Sw1; const _ : fn() = ||
    { __rtic_trait_checks :: implements_rtic_sw_task :: < Sw1 > (); }; impl
    RticSwTask for Sw1
    {
        type SpawnInput = (); fn init(_ : ()) -> Self { Self } fn
        exec(& mut self, _p : ())
        {
            < __rtic_obs as RticObservability > :: on_task_act(TaskId :: Sw1);
            { self.shared().uart.lock(| _uart | { sprintln! ("SW"); }); } <
            __rtic_obs as RticObservability > :: on_task_comp(TaskId :: Sw1);
        } type InitArgs = ();
    } impl Sw1 { pub const fn priority() -> u16 { 2u16 } } impl Sw1
    {
        pub fn shared(& self) -> __sw1_shared_resources
        {
            const TASK_PRIORITY : u16 = 2u16; __sw1_shared_resources ::
            new(TASK_PRIORITY)
        }
    } pub struct __sw1_shared_resources { pub uart : __uart_mutex, } impl
    __sw1_shared_resources
    {
        #[inline(always)] pub fn new(priority : u16) -> Self
        { Self { uart : __uart_mutex :: new(priority), } }
    } impl Sw1
    {
        pub const fn current_core() -> __rtic__internal__Core0
        { unsafe { __rtic__internal__Core0 :: new() } }
    } static mut CORE0_PRIORITY2_DISPATCHER : core :: mem :: MaybeUninit <
    Core0Priority2Dispatcher > = core :: mem :: MaybeUninit :: uninit();
    #[doc(hidden)] pub struct Core0Priority2Dispatcher; const _ : fn() = ||
    {
        __rtic_trait_checks :: implements_rtic_task :: <
        Core0Priority2Dispatcher > ();
    }; impl RticTask for Core0Priority2Dispatcher
    {
        fn init(_ : ()) -> Self { Self } fn exec(& mut self)
        {
            < __rtic_obs as RticObservability > ::
            on_task_act(TaskId :: Core0Priority2Dispatcher);
            {
                unsafe
                {
                    let mut ready_consumer =
                    __rtic_internal__Core0Prio2Tasks__RQ.split().1; while let
                    Some(task) = ready_consumer.dequeue()
                    {
                        match task
                        {
                            Core0Prio2Tasks :: Sw1 =>
                            {
                                let mut input_consumer =
                                __rtic_internal__Sw1__INPUTS.split().1; let input =
                                input_consumer.dequeue_unchecked();
                                SW1.assume_init_mut().exec(input);
                            }
                        }
                    }
                }
            } < __rtic_obs as RticObservability > ::
            on_task_comp(TaskId :: Core0Priority2Dispatcher);
        } type InitArgs = ();
    } impl Core0Priority2Dispatcher
    { pub const fn priority() -> u16 { 2u16 } } impl Core0Priority2Dispatcher
    {
        pub const fn current_core() -> __rtic__internal__Core0
        { unsafe { __rtic__internal__Core0 :: new() } }
    } #[allow(non_snake_case)] #[bsp :: nested_interrupt] fn Timer0Cmp()
    { unsafe { SOME_TASK.assume_init_mut().exec() }; }
    #[allow(non_snake_case)] #[bsp :: nested_interrupt] fn MachineSoft()
    { unsafe { CORE0_PRIORITY2_DISPATCHER.assume_init_mut().exec() }; } pub
    struct __uart_mutex { #[doc(hidden)] task_priority : u16, } impl
    __uart_mutex
    {
        #[inline(always)] pub fn new(task_priority : u16) -> Self
        { Self { task_priority } }
    } impl RticMutex for __uart_mutex
    {
        type ResourceType = ApbUart; fn lock < R >
        (& mut self, f : impl FnOnce(& mut Self :: ResourceType) -> R) -> R
        {
            const CEILING : u16 = 2u16; let task_priority =
            self.task_priority; let resource_ptr = unsafe
            { & mut SHARED.assume_init_mut().uart } as * mut _; unsafe
            {
                rtic :: export ::
                lock(resource_ptr, task_priority as u8, CEILING as u8, f)
            }
        }
    } impl RticReadLock for __uart_mutex
    {
        type ResourceType = ApbUart; fn read_lock < R >
        (& self, f : impl FnOnce(& Self :: ResourceType) -> R) -> R
        {
            const CEILING : u16 = 2u16; let task_priority =
            self.task_priority; let resource_ptr = unsafe
            { & mut SHARED.assume_init_mut().uart } as * mut _; let f = |
            resource : & mut Self :: ResourceType | { f(resource) }; unsafe
            {
                rtic :: export ::
                lock(resource_ptr, task_priority as u8, CEILING as u8, f)
            }
        }
    } pub struct __uart_readable { #[doc(hidden)] task_priority : u16, } impl
    __uart_readable
    {
        #[inline(always)] pub fn new(task_priority : u16) -> Self
        { Self { task_priority } }
    } impl RticReadLock for __uart_readable
    {
        type ResourceType = ApbUart; fn read_lock < R >
        (& self, f : impl FnOnce(& Self :: ResourceType) -> R) -> R
        {
            const CEILING : u16 = 2u16; let task_priority =
            self.task_priority; let resource_ptr = unsafe
            { & mut SHARED.assume_init_mut().uart } as * mut _; let f = |
            resource : & mut Self :: ResourceType | { f(resource) }; unsafe
            {
                rtic :: export ::
                lock(resource_ptr, task_priority as u8, CEILING as u8, f)
            }
        }
    } #[doc = "Unique type for core 0"] pub use core0_type_mod ::
    __rtic__internal__Core0; mod core0_type_mod
    {
        struct __rtic__internal__Core0Inner; pub struct
        __rtic__internal__Core0(__rtic__internal__Core0Inner); impl
        __rtic__internal__Core0
        {
            pub const unsafe fn new() -> Self
            { __rtic__internal__Core0(__rtic__internal__Core0Inner) }
        }
    }
    #[doc =
    r" Type representing tasks that need explicit user initialization"]
    #[doc = r" Entry of "] #[doc = " # CORE 0"] #[bsp :: rt :: entry]
    #[unsafe (no_mangle)] fn main() -> !
    {
        __rtic_interrupt_free(||
        {
            let shared_resources = init(); unsafe
            { SHARED.write(shared_resources); } unsafe
            {
                SOME_TASK.write(SomeTask :: init(()));
                SW1.write(Sw1 :: init(()));
                CORE0_PRIORITY2_DISPATCHER.write(Core0Priority2Dispatcher ::
                init(()));
            } #[cfg(feature = "intc-clic")] bsp :: clic :: Clic ::
            smclicconfig().set_mnlbits(8); unsafe
            { bsp :: register :: mintstatus :: write(0.into()) }; unsafe
            {
                rtic :: export ::
                enable(rtic :: export :: interrupts :: Timer0Cmp, 1u32,); rtic
                :: export ::
                enable(rtic :: export :: interrupts :: MachineSoft, 2u32,);
            }
        }); loop { unsafe { core :: arch :: asm! ("wfi") }; }
    }
    #[doc =
    r" Utility functions used to enforce implementing appropriate task traits"]
    mod __rtic_trait_checks
    {
        use super :: * ; pub fn implements_rtic_sw_task < T : RticSwTask > ()
        {} pub fn implements_rtic_task < T : RticTask > () {}
    }
}