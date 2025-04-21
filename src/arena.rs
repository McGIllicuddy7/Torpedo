use std::{cell::{Ref, RefCell, RefMut, UnsafeCell}, collections::HashSet, ffi::c_void, fmt::Debug, mem::{ManuallyDrop, MaybeUninit}, ops::{Deref, Index, IndexMut}, sync::Mutex};

use libc::pthread_mutex_t;



/*typedef struct ArenaInternal{
	pthread_mutex_t lock;
    char* buffer;
    char* next_ptr;
    char*end;
    char* previous_allocation;
    struct ArenaInternal * next;
	void * defer_queue;
} ArenaInternal;*/
/*typedef struct ArenaInternal_defer{
	void (*func)(void *);
	void * data;
	struct ArenaInternal_defer * next;
}ArenaInternal_defer;*/
#[repr(C)]
struct ArenaInternalDefer{
    func: fn(*mut c_void),
    data:*mut c_void, 
    next:*mut ArenaInternalDefer,
}
#[repr(C)]
struct ArenaInternal{
    lock:libc::pthread_mutex_t,
    buffer:*const u8,
    next_ptr:*const u8, 
    end:*const u8,
    previous_allocation:*const u8, 
    next:Option<Box<ArenaInternal>>,
    defer_que:*mut ArenaInternalDefer,
}


impl Drop for ArenaInternal{
    fn drop(&mut self) {
        unsafe{
            while !self.defer_que.is_null(){ 
                let fc = (*self.defer_que).func;
                fc((*self.defer_que).data);
                self.defer_que = (*self.defer_que).next;
            }
            libc::pthread_mutex_destroy(&mut self.lock as *mut pthread_mutex_t);
            libc::free(self.buffer as *mut c_void);
        }
    }
}
impl ArenaInternal{
    const ALLOC_SZ:usize = 4096*8;
    pub fn new()->ArenaInternal{
        Self::new_sized(Self::ALLOC_SZ)
    }
    pub fn new_sized(size:usize)->ArenaInternal{
        unsafe{
            let mut sz = size;
            if sz %4096 != 0{
                sz = sz+(4096-sz%4096);
            }
            if sz<Self::ALLOC_SZ{
                sz = Self::ALLOC_SZ;
            }
            let mut s:pthread_mutex_t = MaybeUninit::zeroed().assume_init();
            libc::pthread_mutex_init((&mut s) as *mut pthread_mutex_t, std::ptr::null());
            let ptr = libc::malloc(sz) as *mut u8;
            let end = ptr.add(sz);
            Self { lock: s, buffer: ptr, next_ptr: ptr, end, previous_allocation: ptr, next:None, defer_que: std::ptr::null_mut() }
        }
    }
    pub unsafe fn alloc_bytes(&mut self, count:usize)->&mut [u8]{
        unsafe{
        let fia = if count %16 == 0 {count } else {count+(16-count%16)};
        let base:usize = self.next_ptr as usize;
        let end = self.end as usize;
        if fia+base>end {
            if self.next.is_none(){
                let next = ArenaInternal::new_sized(count);
                self.next = Some(Box::new(next));
            } 
            if let Some(n) = &mut self.next{
                n.alloc_bytes(count)
            } else{
                unreachable!()
            }
        } else{
                let out = self.next_ptr;
                self.next_ptr =self.next_ptr.add(fia);
                self.previous_allocation = out;
                let out = std::slice::from_raw_parts_mut(out as *mut u8, count);
                for i in 0..out.len(){
                    out[i] = 0;
                }
                out
        } }
    }
    pub unsafe fn realloc_bytes<'a>(&'a mut self, bytes:&[u8], new_count:usize)->&'a [u8]{
        let out = if bytes.as_ptr() == self.previous_allocation &&new_count >bytes.len(){
            self.next_ptr = self.previous_allocation;
            let out = unsafe{self.alloc_bytes(new_count)};
            out
        } else{
            unsafe{self.alloc_bytes(new_count)}
        };
        let l = if bytes.len()>= new_count{bytes.len()} else{ new_count};
        for i in 0..l{
            out[i] = bytes[i];
        }
        out
    }
    pub unsafe fn alloc_no_drop<T>(&mut self,value:T)->&mut T{
        let out = unsafe {let ptr = self.alloc_bytes(size_of::<T>()).as_ptr()as *mut MaybeUninit<T>;
        assert!(!ptr.is_null());
        assert!(ptr.is_aligned());
        let out = ptr.as_mut().expect("msg"); 
        out.write(value);
        out.assume_init_mut()};
        out
    }
    pub fn alloc<T>(&mut self, value:T)->&mut T{
        let out = unsafe {let ptr = self.alloc_bytes(size_of::<T>()).as_ptr()as *mut MaybeUninit<T>;
        assert!(!ptr.is_null());
        assert!(ptr.is_aligned());
        let out = ptr.as_mut().expect("msg"); 
        out.write(value);
        out.assume_init_mut()};
        out
    }
    pub fn alloc_array<'a,T:Clone> (&'a mut self, value:&[T])->&'a mut[T]{
        unsafe {let ptr = self.alloc_bytes(std::mem::size_of_val(value)).as_ptr()as *mut T;
        let out = std::slice::from_raw_parts_mut(ptr, value.len());
        for i in 0..value.len(){
            out[i] = value[i].clone();
        }
        out
        }
    }
    pub unsafe fn alloc_array_space<T:Clone>(&mut self,count:usize)->MaybeUninit<&mut [T]>{
        unsafe {let ptr = self.alloc_bytes(count*size_of::<T>()).as_ptr()as *mut T;
            let out = MaybeUninit::new(std::slice::from_raw_parts_mut(ptr as *mut T, count));
            out
        }
    }
    pub fn defer_fn(&mut self, func:fn(*mut c_void), data:*mut c_void){
        let mut def = ArenaInternalDefer{func, data,next:std::ptr::null_mut()};
        def.next = self.defer_que;
        let ptr = unsafe{self.alloc_no_drop(def)};
        assert!(!ptr.data.is_null());
        self.defer_que= ptr as *mut ArenaInternalDefer;
    }
    pub fn defer(&mut self,func:Box<dyn FnOnce()> ){
        fn defer_thunk(data:*mut c_void){
            unsafe{
                assert!((data as *const Box<dyn FnOnce()>).is_aligned());
                assert!(!(data).is_null());
                
                let func_ref = (data as *const Box<dyn FnOnce()>).as_ref().expect("msg");
                let func:Box<dyn FnOnce()> = std::mem::transmute_copy(func_ref);
                func();
            }

        }
        let data = unsafe{self.alloc_no_drop(func)} as *const Box<dyn FnOnce()> as *mut c_void;
        assert!(!data.is_null());
        self.defer_fn(defer_thunk, data);
    }
}
#[derive(Debug)]
pub struct Arena{
    internal:UnsafeCell<ArenaInternal>,
    destructors:Mutex<HashSet<*const c_void>>,
}
impl Arena{
    pub fn new()->Box<Self>{
        Box::new(Self{internal:UnsafeCell::new(ArenaInternal::new()), destructors:Mutex::new(HashSet::new())})
    }
    pub fn new_sized(size:usize)->Box<Self>{        
        Box::new(Self{internal:UnsafeCell::new(ArenaInternal::new_sized(size)), destructors:Mutex::new(HashSet::new())})
    }
    unsafe fn lock(&self)->&mut ArenaInternal{
        unsafe{
            let tmp = self.internal.get().as_mut().expect("msg");
            libc::pthread_mutex_lock(&mut tmp.lock as *mut pthread_mutex_t);
            tmp
        }
    }
    unsafe fn unlock(&self){
        unsafe{
            let tmp = self.internal.get().as_mut().expect("msg");
            libc::pthread_mutex_unlock(&mut tmp.lock as *mut pthread_mutex_t);
        }
    }
    pub unsafe fn alloc_bytes(&self, count:usize)->&mut[u8]{
        unsafe {
            let s = self.lock();
            let out = s.alloc_bytes(count);
            self.unlock();
            out
        }
    }
    pub unsafe fn realloc_bytes<'a>(&'a self, bytes:&[u8], new_count:usize)->&'a [u8]{
        unsafe{
            let s = self.lock();
            let out = s.realloc_bytes(bytes,new_count);
            self.unlock();
            out
        }

    }
    pub unsafe fn alloc_no_drop<T>(&self, value:T)->&mut T{
        unsafe {
            let s = self.lock();
            let out = s.alloc_no_drop(value);
            self.unlock();
            out
        }
    }
    pub fn alloc<'a,T>(&'a self, value:T)->&'a mut T{
        unsafe{
            let s = self.lock();
            let out = s.alloc(value);
            self.unlock();
            self.queue_destroy(out);
            out
        }

    }
    pub fn alloc_array<'a, T:Clone> (&'a self, value:&[T])->&'a mut [T]{
        unsafe{
            let s = self.lock();
            let out = s.alloc_array(value);
            self.unlock();
            for i in out.as_ref(){
                self.queue_destroy(i);
            }
            out
        }
    }
    pub unsafe fn alloc_array_space<T:Clone>(&self,count:usize)->MaybeUninit<&mut[T]>{
        unsafe{
            let s = self.lock();
            let out = s.alloc_array_space(count);
            self.unlock();
            assert!(out.as_ptr().is_aligned());
            out
        }
    }
    pub fn defer_fn(&self, func:fn(*mut c_void), data:*mut c_void){
        unsafe {
            let s = self.lock();
            s.defer_fn(func, data);
            self.unlock();
        }
    }
    pub fn defer(&self, func:Box<dyn FnOnce()>){
        unsafe {
            let s = self.lock();
            s.defer(func);
            self.unlock();
        } 
    }
    pub unsafe fn queue_destroy<'a,T>(&'a self, object:&'a T){
        unsafe{
            let p = object as *const T;
            let mut lck = self.destructors.lock().expect("msg");
            if lck.contains(&(p as *const c_void)){
                return;
            }
            lck.insert(p as *const c_void);
            let void = p as *const c_void;
            self.defer(Box::new(move ||{
                let _:T = std::mem::transmute_copy(&*(void as *mut T));
            }));
            drop(lck);
        }
    }
}
unsafe impl Send for Arena{

}
unsafe impl Sync for Arena{

}
pub struct AVec<'a, T:Clone>{
    items: *mut T,
    length:usize,
    capacity:usize,
    arena:&'a Arena,
}
unsafe impl<'a, T:Clone> Send for AVec<'a, T>{

}
unsafe impl<'a, T:Clone> Sync for AVec<'a, T>{

}
impl <'a, T:Clone> Clone for AVec<'a, T>{
    fn clone(&self) -> Self {
        unsafe{
            let new_items :&mut [T] = self.arena.alloc_array_space::<T>(self.capacity).assume_init();
            for i in 0..self.length{
                let _= std::mem::replace(&mut new_items[i], self.get(i).clone());
                self.arena.queue_destroy(self.get(i));
            }
            Self { items:new_items.as_mut_ptr(), length: self.length, capacity:self.capacity, arena: self.arena }
        }
    }
} 
impl <'a, T:Clone> AsRef<[T]> for AVec<'a, T>{
    fn as_ref(&self) -> &[T] {
        if self.length == 0{
            unsafe{
                std::slice::from_raw_parts(std::ptr::dangling(), 0) 
            }
        } else{
            unsafe {std::slice::from_raw_parts(self.items, self.length)}
        }

    }
}
impl <'a, T:Clone> AsMut<[T]> for AVec<'a, T>{
    fn as_mut(&mut self) -> &mut[T] {
        unsafe {std::slice::from_raw_parts_mut(self.items, self.length)}
    }
}
impl <'a, T:Clone> Index<usize> for AVec<'a, T>{
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        return &self.as_ref()[index];
    }
}
impl <'a, T:Clone> IndexMut<usize> for AVec<'a, T>{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        return &mut self.as_mut()[index];
    }
}
impl <'a, T:Clone>Drop for AVec<'a, T>{
    fn drop(&mut self) {
        for i in 0..self.length{
            unsafe{
                let _:T= std::mem::transmute_copy(self.get(i));
            }
        }
    }
}
impl <'a, T:Clone>AVec<'a, T>{
    pub fn new(arena:&'a Arena)->Self{
        Self { items: std::ptr::null_mut(), length:0, capacity: 0, arena }
    }
    pub fn new_with_capacity(arena:&'a Arena, cap:usize)->Self{
        unsafe{
            let items:*mut T = arena.alloc_array_space(cap).assume_init().as_mut_ptr();
            Self { items, length: 0, capacity: cap, arena: arena }
        }
    }
    pub fn get(&self, id:usize)->&T{
        assert!(id<self.length);
        return unsafe{&*(self.items.add(id))}
    }
    pub fn get_mut(&mut self, id:usize)->&mut T{
        assert!(id<self.length);
        return unsafe{&mut *(self.items.add(id))} 
    }
    pub fn push(&mut self,v:T){
        if self.length < self.capacity{
                self.length+=1;
                let _= ManuallyDrop::new(std::mem::replace(self.get_mut(self.length-1),v)); 
        }   else{
            unsafe{
                if self.capacity == 0{
                    self.capacity = 8;
                } 
                self.capacity *= 2;
                let new_items = self.arena.alloc_array_space::<T>(self.capacity).assume_init();
                if new_items.as_ptr() != self.items{
                    for i  in 0..self.length{
                        let _= ManuallyDrop::new(std::mem::replace(&mut new_items[i], self.get(i).clone()));
                    }
                    for i in 0..self.length{
                        let _:T = std::mem::transmute_copy(self.get(i));
                    }
                }
                self.items = new_items.as_mut_ptr();
                self.length+=1;
                let _: ManuallyDrop<T> = ManuallyDrop::new(std::mem::replace(self.get_mut(self.length-1),  v));
            }

        }
    }
    pub fn push_slice(&mut self, v:&[T]){
        for i in v{
            self.push(i.clone());
        }
    }
    pub fn remove(&mut self, id:usize){
        self.length -=1;

        for i in id..self.length-1{
            self[i] = self[i+1].clone();
        }
    }
    pub fn reserve(&mut self, new_cap:usize){
        if self.capacity>new_cap{
            return;
        }
        self.capacity = new_cap;
        unsafe{
            let new_items = self.arena.alloc_array_space::<T>(self.capacity).assume_init();
            if new_items.as_ptr() != self.items{
                for i  in 0..self.length{
                    let _= ManuallyDrop::new(std::mem::replace(&mut new_items[i], self.get(i).clone()));
                }
            }
            self.items = new_items.as_mut_ptr();
        }
    }
}
impl <T:Debug + std::clone::Clone> Debug for AVec<'_,T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut a = f.debug_list();
        for i in self.as_ref(){
            a.entry(i);
        }
        a.finish()
    }
}

#[derive(Clone)]
pub struct AStr<'a>{
    s:&'a str,
    arena:&'a Arena,
}
impl <'a>Deref for AStr<'a>{
    type Target = str;

    fn deref(&self) -> &Self::Target {
        return self.s
    }
}

impl <'a> AStr<'a>{
    pub fn new(arena:&'a Arena, s:&str)->Self{
        let b = s.as_bytes();
        let bytes = unsafe{arena.alloc_bytes(b.len())};
        for i in 0..b.len(){
            bytes[i] = b[i];
        }
        Self { s:unsafe{ std::str::from_utf8_unchecked(bytes)}, arena: arena }
    }
    pub fn extend(&'a self, s:&str)->Self{
        let bytes = unsafe{
             self.arena.alloc_bytes(self.as_bytes().len()+s.as_bytes().len())
        };
        let b0 = self.as_bytes();
        let b1 = s.as_bytes();
        for i in 0..b0.len(){
            bytes[i] = b0[i];
        }
        let l = b0.len();
        for i in 0..b1.len(){
            bytes[i+l] = b1[i];
        }
        let st = std::str::from_utf8_mut(bytes).expect("msg");
        Self { s: st, arena: self.arena }
    }
    pub fn split_by(&self,delim:&str)->AVec<'a,Self>{
        let mut out = AVec::new(&self.arena);
        for i in self.s.split_inclusive(delim){
            if let Some(v) = i.strip_suffix(delim){
                out.push(Self::new(&self.arena,v));
                out.push(Self::new(&self.arena,delim));
            } else{
                out.push(Self::new(&self.arena,i));
            }

        }
        out
    }
}

#[derive(Debug)]
pub struct LLInternal<'a, T>{
    pub data:T, 
    pub next:Option<&'a LLNode<'a, T>>, 
    pub prev:Option<&'a LLNode<'a, T>>, 
}
#[derive(Debug)]
pub struct LLNode<'a,T>{
    pub v:RefCell<LLInternal<'a, T>>,
    pub arena:&'a Arena,
}
impl <'a, T>LLNode<'a,T>{
    pub fn new(arena:&'a Arena,value:T)->&'a Self{
        arena.alloc(Self{v:RefCell::new(LLInternal{data:value, next:None, prev:None,}), arena})
    }
    pub fn get_mut(&'a self)->RefMut<'a, LLInternal<'a, T>>{
        return self.v.borrow_mut();
    }
    pub fn get(&'a self)->Ref<'a, LLInternal<'a, T>>{
        return self.v.borrow();
    }
    pub fn push(&'a self, value:T){
        {
            if let Some(n) = self.get().next{
                n.push(value);            
                return;
            }

        }
        {
            let next = Self::new(&self.arena, value);
            next.get_mut().prev = Some(self);
            self.get_mut().next = Some(next);
        }
    }
    pub fn pop(&'a self){
        let s = self.get();
        if let Some(n) =s.next{
            let t = n.get();
            if t.next.is_some(){
                drop(t);
                n.pop();
            }
            else {
                drop(s);
                self.get_mut().next = None;
            }
        }
    }
}

#[repr(C)]
pub struct GraphInternal<'a, T>{
    pub value:T,
    pub edges:AVec<'a,&'a GraphInternal<'a,T>>,
    pub reached:bool,
}

pub struct GraphNode<'a, T>{
    pub v:std::cell::RefCell<GraphInternal<'a, T>>, 
    pub arena:&'a Arena,
}
impl <'a, T>GraphNode<'a, T>{
    pub fn new(arena:&'a Arena, value:T)->&'a mut Self{
        let base = RefCell::new(GraphInternal{value, edges:AVec::new(arena), reached:false});
        arena.alloc(GraphNode{v:base, arena})
    }
    pub fn get(&'a self)->Ref<'a,GraphInternal<'a, T>>{
        self.v.borrow()
    }
    pub fn get_mut(&'a self)->RefMut<'a, GraphInternal<'a, T>>{
        self.v.borrow_mut()
    }
    pub fn link(&'a self, other:&'a Self){
        
    }
    pub fn unlink(&'a self, other:&'a Self){

    }
}