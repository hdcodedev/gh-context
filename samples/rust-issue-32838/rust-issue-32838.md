# Allocator traits and std::heap

URL: https://github.com/rust-lang/rust/issues/32838

## Body

📢  **This feature has a dedicated working group**, please direct comments and concerns to [the working group's repo](https://github.com/rust-lang/wg-allocators).

The remainder of this post is no longer an accurate summary of the current state; see that dedicated working group instead.

<details>
<summary>Old content</summary>

Original Post:

-----

FCP proposal: https://github.com/rust-lang/rust/issues/32838#issuecomment-336957415
FCP checkboxes: https://github.com/rust-lang/rust/issues/32838#issuecomment-336980230

---

Tracking issue for rust-lang/rfcs#1398 and the `std::heap` module.

- [x] land `struct Layout`, `trait Allocator`, and default implementations in `alloc` crate (https://github.com/rust-lang/rust/pull/42313)
- [x] decide where parts should live (e.g. default impls has dependency on `alloc` crate, but `Layout`/`Allocator` _could_ be in `libcore`...) (https://github.com/rust-lang/rust/pull/42313)
- [ ] fixme from source code: audit default implementations (in `Layout` for overflow errors, (potentially switching to overflowing_add and overflowing_mul as necessary).
- [x] decide if `realloc_in_place` should be replaced with `grow_in_place` and `shrink_in_place` ([comment](https://github.com/rust-lang/rust/issues/32838#issuecomment-208141759)) (https://github.com/rust-lang/rust/pull/42313)
- [ ] review arguments for/against associated error type (see subthread [here](https://github.com/rust-lang/rfcs/pull/1398#issuecomment-204561446))
- [ ] determine what the requirements are on the alignment provided to `fn dealloc`. (See discussion on [allocator rfc](https://github.com/rust-lang/rfcs/pull/1398#issuecomment-198584430) and [global allocator rfc](https://github.com/rust-lang/rfcs/pull/1974#issuecomment-302789872) and  [trait `Alloc` PR](https://github.com/rust-lang/rust/pull/42313#issuecomment-306202489).)
  * Is it required to deallocate with the exact `align` that you allocate with? [Concerns have been raised](https://github.com/rust-lang/rfcs/pull/1974#issuecomment-302789872) that allocators like jemalloc don't require this, and it's difficult to envision an allocator that does require this. ([more discussion](https://github.com/rust-lang/rfcs/pull/1398#issuecomment-198584430)). @ruuda and @rkruppe look like they've got the most thoughts so far on this.
- [ ] should `AllocErr` be `Error` instead? ([comment](https://github.com/rust-lang/rust/pull/42313#discussion_r122580471))
- [x] Is it required to deallocate with the *exact* size that you allocate with? With the `usable_size` business we may wish to allow, for example, that you if you allocate with `(size, align)` you must deallocate with a size somewhere in the range of `size...usable_size(size, align)`. It appears that jemalloc is totally ok with this (doesn't require you to deallocate with a *precise* `size` you allocate with) and this would also allow `Vec` to naturally take advantage of the excess capacity jemalloc gives it when it does an allocation. (although actually doing this is also somewhat orthogonal to this decision, we're just empowering `Vec`). So far @Gankro has most of the thoughts on this. (@alexcrichton believes this was settled in https://github.com/rust-lang/rust/pull/42313 due to the definition of "fits")
- [ ] similar to previous question: Is it required to deallocate with the *exact* alignment that you allocated with? (See comment from [5 June 2017](https://github.com/rust-lang/rust/pull/42313#issuecomment-306202489))
- [x] OSX/`alloc_system` is buggy on *huge* alignments (e.g.  an align of `1 << 32`) https://github.com/rust-lang/rust/issues/30170 #43217
- [ ] should `Layout` provide a `fn stride(&self)` method? (See also https://github.com/rust-lang/rfcs/issues/1397, https://github.com/rust-lang/rust/issues/17027 )
- [x] `Allocator::owns` as a method? https://github.com/rust-lang/rust/issues/44302

State of `std::heap` after https://github.com/rust-lang/rust/pull/42313:

```rust
pub struct Layout { /* ... */ }

impl Layout {
    pub fn new<T>() -> Self;
    pub fn for_value<T: ?Sized>(t: &T) -> Self;
    pub fn array<T>(n: usize) -> Option<Self>;
    pub fn from_size_align(size: usize, align: usize) -> Option<Layout>;
    pub unsafe fn from_size_align_unchecked(size: usize, align: usize) -> Layout;

    pub fn size(&self) -> usize;
    pub fn align(&self) -> usize;
    pub fn align_to(&self, align: usize) -> Self;
    pub fn padding_needed_for(&self, align: usize) -> usize;
    pub fn repeat(&self, n: usize) -> Option<(Self, usize)>;
    pub fn extend(&self, next: Self) -> Option<(Self, usize)>;
    pub fn repeat_packed(&self, n: usize) -> Option<Self>;
    pub fn extend_packed(&self, next: Self) -> Option<(Self, usize)>;
}

pub enum AllocErr {
    Exhausted { request: Layout },
    Unsupported { details: &'static str },
}

impl AllocErr {
    pub fn invalid_input(details: &'static str) -> Self;
    pub fn is_memory_exhausted(&self) -> bool;
    pub fn is_request_unsupported(&self) -> bool;
    pub fn description(&self) -> &str;
}

pub struct CannotReallocInPlace;

pub struct Excess(pub *mut u8, pub usize);

pub unsafe trait Alloc {
    // required
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr>;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);

    // provided
    fn oom(&mut self, _: AllocErr) -> !;
    fn usable_size(&self, layout: &Layout) -> (usize, usize);
    unsafe fn realloc(&mut self,
                      ptr: *mut u8,
                      layout: Layout,
                      new_layout: Layout) -> Result<*mut u8, AllocErr>;
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<*mut u8, AllocErr>;
    unsafe fn alloc_excess(&mut self, layout: Layout) -> Result<Excess, AllocErr>;
    unsafe fn realloc_excess(&mut self,
                             ptr: *mut u8,
                             layout: Layout,
                             new_layout: Layout) -> Result<Excess, AllocErr>;
    unsafe fn grow_in_place(&mut self,
                            ptr: *mut u8,
                            layout: Layout,
                            new_layout: Layout) -> Result<(), CannotReallocInPlace>;
    unsafe fn shrink_in_place(&mut self,
                              ptr: *mut u8,
                              layout: Layout,
                              new_layout: Layout) -> Result<(), CannotReallocInPlace>;

    // convenience
    fn alloc_one<T>(&mut self) -> Result<Unique<T>, AllocErr>
        where Self: Sized;
    unsafe fn dealloc_one<T>(&mut self, ptr: Unique<T>)
        where Self: Sized;
    fn alloc_array<T>(&mut self, n: usize) -> Result<Unique<T>, AllocErr>
        where Self: Sized;
    unsafe fn realloc_array<T>(&mut self,
                               ptr: Unique<T>,
                               n_old: usize,
                               n_new: usize) -> Result<Unique<T>, AllocErr>
        where Self: Sized;
    unsafe fn dealloc_array<T>(&mut self, ptr: Unique<T>, n: usize) -> Result<(), AllocErr>
        where Self: Sized;
}

/// The global default allocator
pub struct Heap;

impl Alloc for Heap {
    // ...
}

impl<'a> Alloc for &'a Heap {
    // ...
}

/// The "system" allocator
pub struct System;

impl Alloc for System {
    // ...
}

impl<'a> Alloc for &'a System {
    // ...
}
```
</details>

## Comments

### Comment 1 by gereeter
_2016-04-11T03:07:19Z_

I unfortunately wasn't paying close enough attention to mention this in the RFC discussion, but I think that `realloc_in_place` should be replaced by two functions, `grow_in_place` and `shrink_in_place`, for two reasons:
- I can't think of a single use case (short of implementing `realloc` or `realloc_in_place`) where it is unknown whether the size of the allocation is increasing or decreasing. Using more specialized methods makes it slightly more clear what is going on.
- The code paths for growing and shrinking allocations tend to be radically different - growing involves testing whether adjacent blocks of memory are free and claiming them, while shrinking involves carving off properly sized subblocks and freeing them. While the cost of a branch inside `realloc_in_place` is quite small, using `grow` and `shrink` better captures the distinct tasks that an allocator needs to perform.

Note that these can be added backwards-compatibly next to `realloc_in_place`, but this would constrain which functions would be by default implemented in terms of which others.

For consistency, `realloc` would probably also want to be split into `grow` and `split`, but the only advantage to having an overloadable `realloc` function that I know of is to be able to use `mmap`'s remap option, which does not have such a distinction.


---

### Comment 2 by gereeter
_2016-04-11T03:12:08Z_

Additionally, I think that the default implementations of `realloc` and `realloc_in_place` should be slightly adjusted - instead of checking against the `usable_size`, `realloc` should just first try to `realloc_in_place`. In turn, `realloc_in_place` should by default check against the usable size and return success in the case of a small change instead of universally returning failure.

This makes it easier to produce a high-performance implementation of `realloc`: all that is required is improving `realloc_in_place`. However, the default performance of `realloc` does not suffer, as the check against the `usable_size` is still performed.


---

### Comment 3 by pnkfelix
_2016-10-26T13:04:27Z_

Another issue: The doc for `fn realloc_in_place` says that if it returns Ok, then one is assured that `ptr` now "fits" `new_layout`.

To me this implies that it must check that the alignment of the given address matches any constraint implied by `new_layout`.

However, I don't think the spec for the underlying `fn reallocate_inplace` function implies that _it_ will perform any such check.
- Furthermore, it seems reasonable that any client diving into using `fn realloc_in_place` will themselves be ensuring that the alignments work (in practice I suspect it means that the same alignment is required everywhere for the given use case...)

So, should the implementation of `fn realloc_in_place` really be burdened with checking that the alignment of the given `ptr` is compatible with that of `new_layout`? It is probably better _in this case_ (of this one method) to push that requirement back to the caller...


---

### Comment 4 by pnkfelix
_2016-10-26T13:05:05Z_

@gereeter you make good points; I will add them to the check list I am accumulating in the issue description.


---

### Comment 5 by pnkfelix
_2016-10-31T17:38:52Z_

(at this point I am waiting for `#[may_dangle]` support to ride the train into the `beta` channel so that I will then be able to use it for std collections as part of allocator integration)


---

### Comment 6 by joshlf
_2017-01-04T20:12:58Z_

I'm new to Rust, so forgive me if this has been discussed elsewhere.

Is there any thought on how to support object-specific allocators? Some allocators such as [slab allocators](http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.29.4759) and [magazine allocators](http://www.parrot.org/sites/www.parrot.org/files/vmem.pdf) are bound to a particular type, and do the work of constructing new objects, caching constructed objects which have been "freed" (rather than actually dropping them), returning already-constructed cached objects, and dropping objects before freeing the underlying memory to an underlying allocator when required.

Currently, this proposal doesn't include anything along the lines of `ObjectAllocator<T>`, but it would be very helpful. In particular, I'm working on an implementation of a magazine allocator object-caching layer (link above), and while I can have this only wrap an `Allocator` and do the work of constructing and dropping objects in the caching layer itself, it'd be great if I could also have this wrap other object allocators (like a slab allocator) and truly be a generic caching layer.

Where would an object allocator type or trait fit into this proposal? Would it be left for a future RFC? Something else?

---

### Comment 7 by Ericson2314
_2017-01-04T20:22:53Z_

I don't think this has been discussed yet.

You could write your own `ObjectAllocator<T>`, and then do `impl<T: Allocator, U> ObjectAllocator<U> for T { .. }`, so that every regular allocator can serve as an object-specific allocator for all objects.

Future work would be modifying collections to use your trait for their nodes, instead of plain ole' (generic) allocators directly.

---

### Comment 8 by nikomatsakis
_2017-01-04T20:25:22Z_

@pnkfelix 

> (at this point I am waiting for #[may_dangle] support to ride the train into the beta channel so that I will then be able to use it for std collections as part of allocator integration)

I guess this has happened?

---

### Comment 9 by joshlf
_2017-01-04T20:27:20Z_

@Ericson2314 Yeah, writing my own is definitely an option for experimental purposes, but I think there'd be much more benefit to it being standardized in terms of interoperability (for example, I plan on also implementing a slab allocator, but it would be nice if a third-party user of my code could use somebody _else's_ slab allocator with my magazine caching layer). My question is simply whether an `ObjectAllocator<T>` trait or something like it is worth discussing. Although it seems that it might be best for a different RFC? I'm not terribly familiar with the guidelines for how much belongs in a single RFC and when things belong in separate RFCs...

---

### Comment 10 by steveklabnik
_2017-01-04T20:42:32Z_

@joshlf 

> Where would an object allocator type or trait fit into this proposal? Would it be left for a future RFC? Something else?

Yes, it would be another RFC.

> I'm not terribly familiar with the guidelines for how much belongs in a single RFC and when things belong in separate RFCs...

that depends on the scope of the RFC itself, which is decided by the person who writes it, and then feedback is given by everyone.

But really, as this is a tracking issue for this already-accepted RFC, thinking about extensions and design changes isn't really for this thread; you should open a new one over on the RFCs repo.

---

### Comment 11 by Ericson2314
_2017-01-04T21:01:36Z_

@joshlf Ah, I thought `ObjectAllocator<T>` was supposed to be a trait. I meant prototype the *trait* not a specific allocator. Yes that trait would merit its own RFC as @steveklabnik says.

------

@steveklabnik yeah now discussion would be better elsewhere. But @joshlf was also raising the issue lest it expose a hitherto unforeseen flaw in the accepted but unimplemented API design. In that sense it matches the earlier posts in this thread.

---

### Comment 12 by joshlf
_2017-01-04T21:27:36Z_

@Ericson2314 Yeah, I thought that was what you meant. I think we're on the same page :)

@steveklabnik Sounds good; I'll poke around with my own implementation and submit an RFC if it ends up seeming like a good idea.

---

### Comment 13 by alexreg
_2017-01-04T21:54:03Z_

@joshlf I don't any reason why custom allocators would go into the compiler or standard library. Once this RFC lands, you could easily publish your own crate that does an arbitrary sort of allocation (even a fully-fledged allocator like jemalloc could be custom-implemented!).

---

### Comment 14 by joshlf
_2017-01-04T21:58:58Z_

@alexreg This isn't about a particular custom allocator, but rather a trait that specifies the type of all allocators which are parametric on a particular type. So just like RFC 1398 defines a trait (`Allocator`) that is the type of any low-level allocator, I'm asking about a trait (`ObjectAllocator<T>`) that is the type of any allocator which can allocate/deallocate and construct/drop objects of type `T`.

---

### Comment 15 by Ericson2314
_2017-01-04T22:01:33Z_

@alexreg See my early point about using *standard library collections* with custom object-specific allocators. 

---

### Comment 16 by alexreg
_2017-01-04T22:02:54Z_

Sure, but I’m not sure that would belong in the standard library. Could easily go into another crate, with no loss of functionality or usability.

> On 4 Jan 2017, at 21:59, Joshua Liebow-Feeser <notifications@github.com> wrote:
> 
> @alexreg <https://github.com/alexreg> This isn't about a particular custom allocator, but rather a trait that specifies the type of all allocators which are parametric on a particular type. So just like RFC 1398 defines a trait (Allocator) that is the type of any low-level allocator, I'm asking about a trait (ObjectAllocator<T>) that is the type of any allocator which can allocate/deallocate and construct/drop objects of type T.
> 
> —
> You are receiving this because you were mentioned.
> Reply to this email directly, view it on GitHub <https://github.com/rust-lang/rust/issues/32838#issuecomment-270499064>, or mute the thread <https://github.com/notifications/unsubscribe-auth/AAEF3IhyyPhFgu1EGHr_GM_Evsr0SRzIks5rPBZGgaJpZM4IDYUN>.
> 



---

### Comment 17 by alexreg
_2017-01-04T22:03:49Z_

I think you’d want to use standard-library collections (any heap-allocated value) with an *arbitrary* custom allocator; i.e. not limited to object-specific ones.

> On 4 Jan 2017, at 22:01, John Ericson <notifications@github.com> wrote:
> 
> @alexreg <https://github.com/alexreg> See my early point about using standard library collections with custom object-specific allocators.
> 
> —
> You are receiving this because you were mentioned.
> Reply to this email directly, view it on GitHub <https://github.com/rust-lang/rust/issues/32838#issuecomment-270499628>, or mute the thread <https://github.com/notifications/unsubscribe-auth/AAEF3CrjYIXqcv8Aqvb4VTyPcajJozICks5rPBbOgaJpZM4IDYUN>.
> 



---

### Comment 18 by joshlf
_2017-01-04T22:13:00Z_

> Sure, but I’m not sure that would belong in the standard library. Could easily go into another crate, with no loss of functionality or usability.

Yes but you probably want some standard library functionality to rely on it (such as what @Ericson2314 suggested).

> I think you’d want to use standard-library collections (any heap-allocated value) with an *arbitrary* custom allocator; i.e. not limited to object-specific ones.

Ideally you'd want both - to accept either type of allocator. There are very significant benefits to using object-specific caching; for example, both slab allocation and magazine caching give very significant performance benefits - take a look at the papers I linked to above if you're curious.

---

### Comment 19 by alexreg
_2017-01-04T22:16:15Z_

But the object allocator trait could simply be a subtrait of the general allocator trait. It’s as simple as that, as far as I’m concerned. Sure, certain types of allocators can be more efficient than general-purpose allocators, but neither the compiler nor the standard really need to (or indeed should) know about this.

> On 4 Jan 2017, at 22:13, Joshua Liebow-Feeser <notifications@github.com> wrote:
> 
> Sure, but I’m not sure that would belong in the standard library. Could easily go into another crate, with no loss of functionality or usability.
> 
> Yes but you probably want some standard library functionality to rely on it (such as what @Ericson2314 <https://github.com/Ericson2314> suggested).
> 
> I think you’d want to use standard-library collections (any heap-allocated value) with an arbitrary custom allocator; i.e. not limited to object-specific ones.
> 
> Ideally you'd want both - to accept either type of allocator. There are very significant benefits to using object-specific caching; for example, both slab allocation and magazine caching give very significant performance benefits - take a look at the papers I linked to above if you're curious.
> 
> —
> You are receiving this because you were mentioned.
> Reply to this email directly, view it on GitHub <https://github.com/rust-lang/rust/issues/32838#issuecomment-270502231>, or mute the thread <https://github.com/notifications/unsubscribe-auth/AAEF3L9F9r_0T5evOtt7Es92vw6gBxR9ks5rPBl9gaJpZM4IDYUN>.
> 



---

### Comment 20 by joshlf
_2017-01-04T22:28:41Z_

> But the object allocator trait could simply be a subtrait of the general allocator trait. It’s as simple as that, as far as I’m concerned. Sure, certain types of allocators can be more efficient than general-purpose allocators, but neither the compiler nor the standard really need to (or indeed should) know about this.

Ah, so the problem is that the semantics are different. `Allocator` allocates and frees raw byte blobs. `ObjectAllocator<T>`, on the other hand, would allocate already-constructed objects and would also be responsible for dropping these objects (including being able to cache constructed objects which could be handed out later in leu of constructing a newly-allocated object, which is expensive). The trait would look something like this:

```
trait ObjectAllocator<T> {
    fn alloc() -> T;
    fn free(t T);
}
```

This is not compatible with `Allocator`, whose methods deal with raw pointers and have no notion of type. Additionally, with `Allocator`s, it is the caller's responsibility to `drop` the object being freed first. This is really important - knowing about the type `T` allows `ObjectAllocator<T>` to do things like call `T`'s `drop` method, and since `free(t)` moves `t` into `free`, the caller _cannot_ drop `t` first - it is instead the `ObjectAllocator<T>`'s responsibility. Fundamentally, these two traits are incompatible with one another.

---

### Comment 21 by alexreg
_2017-01-05T00:16:00Z_

Ah right, I see. I thought this proposal already included something like that, i.e. a “higher-level” allocator over the byte level. In that case, a perfectly fair proposal!

> On 4 Jan 2017, at 22:29, Joshua Liebow-Feeser <notifications@github.com> wrote:
> 
> But the object allocator trait could simply be a subtrait of the general allocator trait. It’s as simple as that, as far as I’m concerned. Sure, certain types of allocators can be more efficient than general-purpose allocators, but neither the compiler nor the standard really need to (or indeed should) know about this.
> 
> Ah, so the problem is that the semantics are different. Allocator allocates and frees raw byte blobs. ObjectAllocator<T>, on the other hand, would allocate already-constructed objects and would also be responsible for dropping these objects (including being able to cache constructed objects which could be handed out later in leu of constructing a newly-allocated object, which is expensive). The trait would look something like this:
> 
> trait ObjectAllocator<T> {
>     fn alloc() -> T;
>     fn free(t T);
> }
> This is not compatible with Allocator, whose methods deal with raw pointers and have no notion of type. Additionally, with Allocators, it is the caller's responsibility to drop the object being freed first. This is really important - knowing about the type T allows ObjectAllocator<T> to do things like call T's drop method, and since free(t) moves t into free, the caller cannot drop t first - it is instead the ObjectAllocator<T>'s responsibility. Fundamentally, these two traits are incompatible with one another.
> 
> —
> You are receiving this because you were mentioned.
> Reply to this email directly, view it on GitHub <https://github.com/rust-lang/rust/issues/32838#issuecomment-270505704>, or mute the thread <https://github.com/notifications/unsubscribe-auth/AAEF3GViJBefuk8IWgPauPyL5tV78Fn5ks5rPB08gaJpZM4IDYUN>.
> 



---

### Comment 22 by joshlf
_2017-01-05T00:53:18Z_

@alexreg Ah yes, I was hoping so too :) Oh well - it'll have to wait for another RFC.

---

### Comment 23 by alexreg
_2017-01-05T00:55:05Z_

Yes, do kick start that RFC, I’m sure it would get plenty of support! And thanks for the clarification (I hadn’t kept up with the details of this RFC at all).

> On 5 Jan 2017, at 00:53, Joshua Liebow-Feeser <notifications@github.com> wrote:
> 
> @alexreg <https://github.com/alexreg> Ah yes, I was hoping so too :) Oh well - it'll have to wait for another RFC.
> 
> —
> You are receiving this because you were mentioned.
> Reply to this email directly, view it on GitHub <https://github.com/rust-lang/rust/issues/32838#issuecomment-270531535>, or mute the thread <https://github.com/notifications/unsubscribe-auth/AAEF3MQQeXhTliU5CBsoheBFL26Ee9WUks5rPD8RgaJpZM4IDYUN>.
> 



---

### Comment 24 by burdges
_2017-01-12T18:13:02Z_

A crate for testing custom allocators would be useful. 

---

### Comment 25 by hawkw
_2017-02-07T17:35:14Z_

Forgive me if I'm missing something obvious, but is there a reason for the `Layout` trait described in this RFC not to implement `Copy` as well as `Clone`, since it's just POD?

---

### Comment 26 by Ericson2314
_2017-03-20T19:59:46Z_

I can't think of any.

---

### Comment 27 by joshlf
_2017-04-05T23:15:35Z_

Sorry to be bringing this up so late in the process, but...

Might it be worth adding support for a `dealloc`-like function that isn't a method, but rather a function? The idea would be to use alignment to be able to infer from a pointer where in memory its parent allocator is and thus be able to free without needing an explicit allocator reference.

This could be a big win for data structures that use custom allocators. This would allow them to not keep a reference to the allocator itself, but rather only need to be parametric on the _type_ of the allocator to be able to call the right `dealloc` function. For example, if `Box` is eventually modified to support custom allocators, then it'd be able to keep being only a single word (just the pointer) as opposed to having to be expanded to two words to store a reference to the allocator as well.

On a related note, it might also be useful to support a non-method `alloc` function to allow for global allocators. This would compose nicely with a non-method `dealloc` function - for global allocators, there'd be no need to do any kind of pointer-to-allocator inference since there'd only be a single static instance of the allocator for the whole program.

---

### Comment 28 by eddyb
_2017-04-06T15:20:24Z_

@joshlf The current design allows you to get that by just having your allocator be a (zero-sized) unit type - i.e. `struct MyAlloc;` that you then implement the `Allocator` trait on.
Storing references or nothing at all, *always*, is *less general* than storing the allocator by-value.

---

### Comment 29 by joshlf
_2017-04-06T16:17:36Z_

I could see that being true for a directly-embedded type, but what about if a data structure decies to keep a reference instead? Does a reference to a zero-sized type take up zero space? That is, if I have:

```rust
struct Foo()

struct Blah{
    foo: &Foo,
}
```

Does `Blah` have zero size?

---

### Comment 30 by joshlf
_2017-04-06T16:21:08Z_

Actually, even it's possible, you might not want your allocator to have zero size. For example, you might have an allocator with a non-zero size that you allocate _from_, but that has the ability to free objects w/o knowing about the original allocator. This would still be useful for making a `Box` take only a word. You'd have something like `Box::new_from_allocator` which would have to take an allocator as an argument - and it might be a nonzero-sized allocator - but if the allocator supported freeing without the original allocator reference, the returned `Box<T>` could avoid storing a reference to the allocator that was passed in the original `Box::new_from_allocator` call.

---

### Comment 31 by Ericson2314
_2017-04-06T17:49:19Z_

> For example, you might have an allocator with a non-zero size that you allocate from, but that has the ability to free objects w/o knowing about the original allocator.

I recall long, long, ago proposing factoring out separate allocator and deallocator traits (with associate types connecting the two) for basically this reason.

---

### Comment 32 by Zoxc
_2017-04-06T18:14:37Z_

Is/should the compiler be allowed to optimize away allocations with these allocators?

---

### Comment 33 by joshlf
_2017-04-06T20:30:49Z_

> Is/should the compiler be allowed to optimize away allocations with these allocators?

@Zoxc What do you mean?

> I recall long, long, ago proposing factoring out separate allocator and deallocator traits (with associate types connecting the two) for basically this reason.

For posterity, let me clarify this statement (I talked to @Ericson2314 about it offline): The idea is that a `Box` could be parametric just on a deallocator. So you could have the following implementation:

```rust
trait Allocator {
    type D: Deallocator;
    
    fn get_deallocator(&self) -> Self::D;
}

trait Deallocator {}

struct Box<T, D: Deallocator> {
    ptr: *mut T,
    d: D,
}

impl<T, D: Deallocator> Box<T, D> {
    fn new_from_allocator<A: Allocator>(x: T, a: A) -> Box<T, A::D> {
        ...
        Box {
            ptr: ptr,
            d: a.get_deallocator()
        }
    }
}
```

This way, when calling `new_from_allocator`, if `A::D` is a zero-sized type, then the `d` field of `Box<T, A::D>` takes up zero size, and so the size of the resulting `Box<T, A::D>` is a single word.

---

### Comment 34 by joshlf
_2017-05-05T17:40:24Z_

Is there a timeline for when this will land? I'm working on some allocator stuff, and it'd be nice if this stuff were there for me to build off of.

If there's interest, I'd be happy to lend some cycles to this, but I'm relatively new to Rust, so that might just create more work for the maintainers in terms of having to code review a newbie's code. I don't want to step on anybody's toes, and I don't want to make more work for people.

---

### Comment 35 by alexcrichton
_2017-05-19T16:52:25Z_

Ok we've recently [met to evaluate the state of allocators](https://github.com/rust-lang/rust/issues/32838) and I think there's some good news for this as well! It looks like support has not yet landed in libstd for these APIs, but everyone's still happy with them landing at any time!

One thing we discussed is that changing over all the libstd types may be a bit premature due to possible inference issues, but regardless of that it seems like a good idea to land the `Allocator` trait and the `Layout` type in the proposed `std::heap` module for experimentation elsewhere in the ecosystem!

@joshlf if you'd like to help out here I think that'd be more than welcome! The first piece will likely be landing the basic type/trait from this RFC into the standard library, and then from there we can start experimenting and playing around with collections in libstd as well.

---

### Comment 36 by Ericson2314
_2017-05-19T21:55:02Z_

@alexcrichton I think your link is broken? It points back here.

> One thing we discussed is that changing over all the libstd types may be a bit premature due to possible inference issues

Adding the trait is a good first step, but without refactoring existing APIs to use it they won't see much usage. In https://github.com/rust-lang/rust/issues/27336#issuecomment-300721558 I propose we can refactor the crates behind the facade immediately, but add newtype wrappers in `std`. Annoying to do, but allows us to make progress.

---

### Comment 37 by joshlf
_2017-05-19T22:37:18Z_

@alexcrichton What would be the process for getting object allocators in? My experiments so far (soon to be public; I can add you to the private GH repo if you're curious) and the discussion [here](https://github.com/rust-lang/rfcs/pull/1974) have led me to believe that there's going to be a near-perfect symmetry between the allocator traits and object allocator traits. E.g., you'll have something like (I changed `Address` to `*mut u8` for symmetry with `*mut T` from `ObjectAllocator<T>`; we'd probably end up with `Address<T>` or something like that):

```rust
unsafe trait Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr>;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
}
```

```rust
unsafe trait ObjectAllocator<T> {
    unsafe fn alloc(&mut self) -> Result<*mut T, AllocErr>;
    unsafe fn dealloc(&mut self, ptr: *mut T);
}
```

Thus, I think experimenting with both allocators and object allocators at the same time could be useful. I'm not sure if this is the right place for that, though, or whether there should be another RFC or, at the very least, a separate PR.

---

### Comment 38 by alexcrichton
_2017-05-20T05:02:56Z_

Oh I meant to link [here](https://github.com/rust-lang/rust/issues/27700#issuecomment-302754526) which also has information about the global allocator. @joshlf is that what you're thinking?

---

### Comment 39 by pnkfelix
_2017-05-30T14:35:49Z_

It sounds like @alexcrichton wants a PR that provides the `Allocator` trait and `Layout` type, even if its not integrated into any collection in `libstd`.

If I understand that correctly, then I can put up a PR for that. I had not done so because I keep trying to get at least integration with `RawVec` and `Vec` prototyped. (At this point I have `RawVec` done, but `Vec` is a bit more challenging due to the many other structures that build off of it, like `Drain` and `IntoIter` etc...)

---

### Comment 40 by pnkfelix
_2017-05-30T14:50:29Z_

actually, my current branch seems like it might actually build (and the one test for integration with`RawVec` passed), so I went ahead and posted it: #42313

---

### Comment 41 by pnkfelix
_2017-05-30T14:58:05Z_

@hawkw asked: 
> Forgive me if I'm missing something obvious, but is there a reason for the Layout trait described in this RFC not to implement Copy as well as Clone, since it's just POD?

The reason I made `Layout` only implement `Clone` and not `Copy` is that I wanted to leave open the possibility of adding more structure to the `Layout` type. In particular, I *still* am interested in trying to have the `Layout` attempt to track any type structure used to construct it (e.g. 16-array of `struct { x: u8, y: [char; 215] }`), so that allocators would have the option of exposing instrumentation routines that report on what types their current contents are composes from.

This would almost certainly have to be an optional feature, i.e. it seems like the tide is firmly against forcing developers to to use the type-enriched `Layout` constructors. so any instrumentation of this form would need to include something like an "unknown memory blocks" category to handle allocations that do not have the type information.

But nonetheless, features like this were the main reason why I did not opt to make `Layout` implement `Copy`; I basically figured that an implementation of `Copy` would be a premature constraint on `Layout` itself.

---

### Comment 42 by joshlf
_2017-06-06T04:00:12Z_

@alexcrichton @pnkfelix 

Looks like @pnkfelix has this one covered, and that PR is getting traction, so let's just go with that. I'm looking over it and making comments now, and it looks great!

---

### Comment 43 by alexcrichton
_2017-06-06T18:59:51Z_

Currently the signature for `Allocator::oom` is:

```rust
    fn oom(&mut self, _: AllocErr) -> ! {
        unsafe { ::core::intrinsics::abort() }
    }
```

It was [brought to my attention](https://bugzilla.mozilla.org/show_bug.cgi?id=1369424), though, that Gecko at least likes to know the allocation size as well on OOM. We may wish to consider this when stabilizing to perhaps add context like `Option<Layout>` for *why* OOM is happening.

---

### Comment 44 by joshlf
_2017-06-06T19:13:53Z_

@alexcrichton 
Might it be worth having either multiple `oom_xxx` variants or an enum of different argument types? There are a couple of different signatures for methods that could fail (e.g., `alloc` takes a layout, `realloc` takes a pointer, an original layout, and a new layout, etc), and there might be cases in which an `oom`-like method would want to know about all of them.

---

### Comment 45 by alexcrichton
_2017-06-06T22:21:32Z_

@joshlf that's true, yes, but I'm not sure if it's useful. I wouldn't want to just add features because we can, they should continue to be well motivated.

---

A point for stabilization here is also "determine what the requirements are on the alignment provided to `fn dealloc`", and the [current implementation](https://github.com/rust-lang/rust/blob/a032cb89c5d9b436c1c57f8a6d5961d898f5c2b6/src/liballoc_system/lib.rs#L263-L272) of `dealloc` on Windows uses `align` to determine how to correctly free. @ruuda you may be interested in this fact.

---

### Comment 46 by ruuda
_2017-06-07T10:41:16Z_

> A point for stabilization here is also "determine what the requirements are on the alignment provided to `fn dealloc`", and the current implementation of `dealloc` on Windows uses `align` to determine how to correctly free.

Yes, I think this is how I initially ran into this; my program crashed on Windows because of this. As `HeapAlloc` makes no alignment guarantees, `allocate` allocates a bigger region and stores the original pointer in a header, but as an optimization this is avoided if the alignment requirements would be satisfied anyway. I wonder if there is a way to convert `HeapAlloc` into an alignment-aware allocator that does not require alignment on free, without losing this optimization.

---

### Comment 47 by retep998
_2017-06-07T11:23:18Z_

@ruuda 

>As `HeapAlloc` makes no alignment guarantees

It does provide a minimum alignment guarantee of 8 bytes for 32bit or 16 bytes for 64bit, it just doesn't provide any way to guarantee alignment higher than that.

The `_aligned_malloc` provided by the CRT on Windows can provide allocations of higher alignment, but notably it *must* be paired with `_aligned_free`, using `free` is illegal. So if you don't know whether an allocation was done via `malloc` or `_aligned_malloc` then you're stuck in the same conundrum that `alloc_system` is in on Windows if you don't know the alignment for `deallocate`. The CRT does **not** provide the standard `aligned_alloc` function which can be paired with `free`, so even Microsoft hasn't been able to solve this problem. (Although it *is* a C11 function and Microsoft doesn't support C11 so that's a weak argument.)

Do note that `deallocate` only cares about the alignment to know whether it is overaligned, the actual value itself is irrelevant. If you wanted a `deallocate` that was truly alignment independent, you could simply treat *all* allocations as overaligned, but you'd waste a lot of memory on small allocations.

---

### Comment 48 by pnkfelix
_2017-06-07T15:42:17Z_

@alexcrichton [wrote](https://github.com/rust-lang/rust/issues/32838#issuecomment-306584529):

> Currently the signature for `Allocator::oom` is:
> 
> ```rust
>     fn oom(&mut self, _: AllocErr) -> ! {
>         unsafe { ::core::intrinsics::abort() }
>     }
> ```
> 
> It was [brought to my attention](https://bugzilla.mozilla.org/show_bug.cgi?id=1369424), though, that Gecko at least likes to know the allocation size as well on OOM. We may wish to consider this when stabilizing to perhaps add context like `Option<Layout>` for *why* OOM is happening.

The `AllocErr` [already carries](https://github.com/pnkfelix/rust/blob/3b5b95c9fb0438f240c468de9062d81da0782f73/src/liballoc/allocator.rs#L268) the `Layout` in the `AllocErr::Exhausted` variant. We could just add the `Layout` to the `AllocErr::Unsupported` variant as well, which I think would be simplest in terms of client expectations. (It does have the drawback of silghtly increasing the side of the `AllocErr` enum itself, but maybe we shouldn't worry about that...)

---

### Comment 49 by alexcrichton
_2017-06-07T15:45:12Z_

Oh I suspect that's all that's needed, thanks for the correction @pnkfelix!

---

### Comment 50 by alexcrichton
_2017-06-20T14:37:59Z_

I'm going to start repurposing this issue for the tracking issue for `std::heap` in general as it will be after https://github.com/rust-lang/rust/pull/42727 lands. I'll be closing a few other related issues in favor of this.

---

### Comment 51 by Ericson2314
_2017-06-20T14:52:39Z_

Is there a tracking issue for converting collections? Now that the PRs are merged, I'd like to

 - Discuss the associated error type
 - Discuss converting collections to use any local allocator, (especially leveraging associated error type)

---

### Comment 52 by alexcrichton
_2017-06-20T15:11:43Z_

I've opened https://github.com/rust-lang/rust/issues/42774 to track integration of `Alloc` into std collections. With historical discussion in the libs team that's likely to be on a separate track of stabilization than an initial pass of the `std::heap` module.

---

### Comment 53 by alexcrichton
_2017-06-20T17:22:16Z_

While reviewing allocator-related issues I also came across https://github.com/rust-lang/rust/issues/30170 which @pnkfelix awhile back. It looks like the OSX system allocator is buggy with high alignments and when running that program with jemalloc it's segfaulting during deallocation on Linux at least. Worth considering during stabilization!

---

### Comment 54 by pnkfelix
_2017-06-21T11:49:42Z_

I've opened #42794 as a place to discuss the specific question of whether zero-sized allocations need to match their requested alignment.

---

### Comment 55 by pnkfelix
_2017-06-21T11:50:34Z_

(oh wait, zero-sized allocations are *illegal* in user allocators!)

---

### Comment 56 by SimonSapin
_2017-07-08T08:33:00Z_

Since the `alloc::heap::allocate` function and friends are now gone in Nightly, I’ve updated Servo to use this new API. This is part of the diff:

```diff
-use alloc::heap;
+use alloc::allocator::{Alloc, Layout};
+use alloc::heap::Heap;
```
```diff
-        let ptr = heap::allocate(req_size as usize, FT_ALIGNMENT) as *mut c_void;
+        let layout = Layout::from_size_align(req_size as usize, FT_ALIGNMENT).unwrap();
+        let ptr = Heap.alloc(layout).unwrap() as *mut c_void;
```

I feel the ergonomics are not great. We went from importing one item to importing three from two different modules.

* Would it make sense to have a convenience method for `allocator.alloc(Layout::from_size_align(…))`?
* Would it make sense to make `<Heap as Alloc>::_` methods available as free functions or inherent methods? (To have one fewer item to import, the `Alloc` trait.)

---

### Comment 57 by SimonSapin
_2017-07-08T08:33:54Z_

Alternatively, could the `Alloc` trait be in the prelude or is it too niche of a use case?

---

### Comment 58 by eddyb
_2017-07-08T08:38:10Z_

@SimonSapin IMO there isn't much of a point in optimizing the ergonomics of such a low-level API.

---

### Comment 59 by joshlf
_2017-07-08T16:26:54Z_

@SimonSapin 

> I feel the ergonomics are not great. We went from importing one item to importing three from two different modules.

I had the exact same feeling with my codebase - it's pretty clunky now.

> Would it make sense to have a convenience method for `allocator.alloc(Layout::from_size_align(…))?`

Do you mean in the `Alloc` trait, or just for `Heap`? One thing to consider here is that there's now a third error condition: `Layout::from_size_align` returns an `Option`, so it could return `None` in addition to the normal errors you can get when allocating.

> Alternatively, could the `Alloc` trait be in the prelude or is it too niche of a use case?

> IMO there isn't much of a point in optimizing the ergonomics of such a low-level API.

I agree that it's probably too low-level to put in the prelude, but I still think that there's value in optimizing the ergonomics (selfishly, at least - that was a really annoying refactor 😝 ).

---

### Comment 60 by alexcrichton
_2017-07-08T18:27:59Z_

@SimonSapin did you not handle OOM before? Also in `std` all three types are available in the `std::heap` module (they're supposed to be in one module). Also did you not handle the case of overflowing sizes before? Or zero-sized types?

---

### Comment 61 by SimonSapin
_2017-07-08T18:55:37Z_

> did you not handle OOM before?

When it existed the `alloc::heap::allocate` function returned a pointer without a `Result` and did not leave a choice in OOM handling. I think it aborted the process. Now I’ve added `.unwrap()` to panic the thread.

> they're supposed to be in one module

I see now that `heap.rs` contains `pub use allocator::*;`. But when I clicked `Alloc` in the impl listed on the rustdoc page for `Heap` I was sent to `alloc::allocator::Alloc`.

As to the rest, I haven’t looked into it. I’m porting to a new compiler a big pile of code that was written years ago. I think these are callbacks for FreeType, a C library.

---

### Comment 62 by retep998
_2017-07-09T00:20:27Z_

>When it existed the alloc::heap::allocate function returned a pointer without a Result and did not leave a choice in OOM handling.

It did give you a choice. The pointer it returned could have been a null pointer which would indicate the heap allocator failed to allocate. This is why I'm so glad it switched to `Result` so people don't forget to handle that case.

---

### Comment 63 by SimonSapin
_2017-07-09T06:24:48Z_

Oh well, maybe the FreeType ended up doing a null check, I don’t know. Anyway, yes, returning a Result is good.

---

### Comment 64 by pnkfelix
_2017-07-13T11:05:53Z_

Given #30170 and #43097, I am tempted to resolve the OS X issue with ridiculously big alignments by simply specifying that users *cannot* request alignments >= `1 << 32`.

One very easy way to enforce this: Change the [`Layout` interface](https://doc.rust-lang.org/nightly/std/heap/struct.Layout.html) so that `align` is denoted by a `u32` instead of a `usize`.

@alexcrichton do you have thoughts on this? Should I just make a PR that does this?

---

### Comment 65 by SimonSapin
_2017-07-13T11:22:47Z_

@pnkfelix `Layout::from_size_align` would still take `usize` and return an error on `u32` overflow, right?

---

### Comment 66 by pnkfelix
_2017-07-13T11:29:14Z_

@SimonSapin what reason is there to have it continue taking `usize` align, if a *static* precondition is that it is unsafe to pass a value >= `1 << 32` ?

---

### Comment 67 by pnkfelix
_2017-07-13T11:31:30Z_

and if the answer is "well some allocators might support an alignment >= `1 << 32`", then we're back to the status quo and you can disregard my suggestion. The point of my suggestion is basically a "+1" to comments like [this one](https://github.com/rust-lang/rust/issues/30170#issuecomment-161437020)

---

### Comment 68 by SimonSapin
_2017-07-13T11:40:38Z_

Because `std::mem::align_of` returns `usize`

---

### Comment 69 by pnkfelix
_2017-07-13T11:42:57Z_

@SimonSapin ah, good old stable API's... sigh.

---

### Comment 70 by alexcrichton
_2017-07-13T14:22:58Z_

@pnkfelix limiting to `1 << 32` seems reasonable to me!

---

### Comment 71 by alexcrichton
_2017-10-16T17:16:25Z_

@rfcbot fcp merge

Ok this trait and its types have bake for awhile now and also been the underlying implementation of the standard collections since its inception. I would propose starting out with a particularly conservative initial offering, namely only stabilizing the following interface:

```rust
pub struct Layout { /* ... */ }

extern {
    pub type void;
}

impl Layout {
    pub fn from_size_align(size: usize, align: usize) -> Option<Layout>;
    pub unsafe fn from_size_align_unchecked(size: usize, align: usize) -> Layout;

    pub fn size(&self) -> usize;
    pub fn align(&self) -> usize;
}

pub unsafe trait Alloc {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut void;
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> *mut void;
    unsafe fn dealloc(&mut self, ptr: *mut void, layout: Layout);

    // all other methods are default and unstable
}

/// The global default allocator
pub struct Heap;

impl Alloc for Heap {
    // ...
}

impl<'a> Alloc for &'a Heap {
    // ...
}

/// The "system" allocator
pub struct System;

impl Alloc for System {
    // ...
}

impl<'a> Alloc for &'a System {
    // ...
}
```

<details><summary>Original proposal</summary><p>

```rust
pub struct Layout { /* ... */ }

impl Layout {
    pub fn from_size_align(size: usize, align: usize) -> Option<Layout>;
    pub unsafe fn from_size_align_unchecked(size: usize, align: usize) -> Layout;

    pub fn size(&self) -> usize;
    pub fn align(&self) -> usize;
}

// renamed from AllocErr today
pub struct Error {
    // ...
}

impl Error {
    pub fn oom() -> Self;
}

pub unsafe trait Alloc {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, Error>;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);

    // all other methods are default and unstable
}

/// The global default allocator
pub struct Heap;

impl Alloc for Heap {
    // ...
}

impl<'a> Alloc for &'a Heap {
    // ...
}

/// The "system" allocator
pub struct System;

impl Alloc for System {
    // ...
}

impl<'a> Alloc for &'a System {
    // ...
}
```
</p></details>

Notably:

* Only stabilizing `alloc`, `alloc_zeroed`, and `dealloc` methods on the `Alloc` trait for now. I think this solves the most pressing issue we have today, defining a custom global allocator.
* Remove the `Error` type in favor of just using raw pointers.
* Change the `u8` type in the interface to `void`
* A stripped down version of the `Layout` type.

There are still open questions such as what to do with `dealloc` and alignment (precise alignment? fits? unsure?), but I'm hoping we can resolve them during FCP as it likely won't be an API-breaking change.

---

### Comment 72 by joshlf
_2017-10-16T17:51:18Z_

+1 to getting something stabilized!

> Renames `AllocErr` to `Error` and moves the interface to be a bit more conservative.

Does this eliminate the option for allocators to specify `Unsupported`? At the risk of harping more on something I've been harping on a lot, I think that #44557 is still an issue.

> `Layout`

It looks like you've removed some of the methods from `Layout`. Did you mean to have the ones you left out actually removed, or just left as unstable?

---

### Comment 73 by SimonSapin
_2017-10-16T18:15:37Z_

> ```rust
> impl Error {
>     pub fn oom() -> Self;
> }
> ```

Is this a constructor for what’s today `AllocErr::Exhausted`? If so, shouldn’t it have a `Layout` parameter?

---

### Comment 74 by rfcbot
_2017-10-16T18:16:41Z_

Team member @alexcrichton has proposed to merge this. The next step is review by the rest of the tagged teams:

* [x] @BurntSushi
* [x] @Kimundi
* [x] @alexcrichton
* [x] @aturon
* [x] @cramertj
* [x] @dtolnay
* [x] @eddyb
* [x] @nikomatsakis
* [x] @nrc
* [x] @pnkfelix
* [x] @sfackler
* [x] @withoutboats

Concerns:

* ~~*mut u8~~ resolved by https://github.com/rust-lang/rust/issues/32838#issuecomment-342622672

Once these reviewers reach consensus, this will enter its final comment period. If you spot a major issue that hasn't been raised at any point in this process, please speak up!

See [this document](https://github.com/dikaiosune/rfcbot-rs/blob/master/RFCBOT.md) for info about what commands tagged team members can give me.

---

### Comment 75 by cramertj
_2017-10-16T19:00:13Z_

I'm really excited about getting to stabilize some of this work!

One question: in the above thread, @joshlf and @Ericson2314 [raised an interesting point](https://github.com/rust-lang/rust/issues/32838#issuecomment-292254121) about the possibility of separating the `Alloc` and `Dealloc` traits in order to optimize for cases in which `alloc` requires some data, but `dealloc` requires no extra information, so the `Dealloc` type can be zero-sized.

Was this question ever resolved? What are the disadvantages of separating the two traits?

---

### Comment 76 by alexcrichton
_2017-10-16T19:05:14Z_

@joshlf 

> Does this eliminate the option for allocators to specify Unsupported? 

Yes and no, it would mean that such an operation is not supported in *stable* rust immediately, but we could continue to support it in unstable Rust.


> Did you mean to have the ones you left out actually removed, or just left as unstable?

Indeed! Again though I'm just proposing a stable API surface area, we can leave all the other methods as unstable. Over time we can continue to stabilize more of the functionality. I think it's best to start as conservative as we can.

---

@SimonSapin 

> Is this a constructor for what’s today AllocErr::Exhausted? If so, shouldn’t it have a Layout parameter?

Aha good point! I sort of wanted to leave the possibility though to make `Error` a zero-sized type if we really needed it, but we can of course keep the layout-taking methods in unstable and stabilize them if necessary. Or do you think that the layout-preserving `Error` should be stabilized in the first pass?

---

@cramertj 

I hadn't personally seen such a question/concern yet (I think I missed it!), but I wouldn't personally see it as being worth it. Two traits is twice the boilerplate in general, as now everyone would have to type `Alloc + Dealloc` in collections for example. I would expect that such a specialized use would not want to inform the interface all other users end up using, personally.

---

### Comment 77 by joshlf
_2017-10-16T19:41:52Z_

@cramertj @alexcrichton 

> I hadn't personally seen such a question/concern yet (I think I missed it!), but I wouldn't personally see it as being worth it.

In general I agree that it's not worth it with one glaring exception: `Box`. `Box<T, A: Alloc>` would, given the current definition of `Alloc`, have to be at least two words large (the pointer that it already has and a reference to an `Alloc` at the very least) except in the case of global singletons (which can be implemented as ZSTs). A 2x (or more) blowup in the space required to store such a common and fundamental type is concerning to me.

---

### Comment 78 by cramertj
_2017-10-16T20:38:13Z_

@alexcrichton 
> as now everyone would have to type Alloc + Dealloc in collections for example

We could add something like this:
```rust
trait Allocator: Alloc + Dealloc {}
impl<T> Allocator for T where T: Alloc + Dealloc {}
```

---

### Comment 79 by SimonSapin
_2017-10-16T21:03:19Z_

> A 2x (or more) blowup in the space required to store such a common and fundamental type

Only when you use a custom allocator that is not process-global. `std::heap::Heap` (the default) is zero-size.

---

### Comment 80 by SimonSapin
_2017-10-16T21:07:44Z_

> Or do you think that the layout-preserving Error should be stabilized in the first pass?

@alexcrichton I don’t really understand why this proposed first pass is at it is at all. There’s barely more than could already be done by abusing `Vec`, and not enough for example to use https://crates.io/crates/jemallocator.

What still needs to be resolved to stabilize the whole thing?

---

### Comment 81 by joshlf
_2017-10-16T21:31:45Z_

> Only when you use a custom allocator that is not process-global. std::heap::Heap (the default) is zero-size.

That seems like the primary use case of having parametric allocators, no? Imagine the following simple definition of a tree:

```rust
struct Node<T, A: Alloc> {
    t: T,
    left: Option<Box<Node<T, A>>>,
    right: Option<Box<Node<T, A>>>,
}
```

A tree constructed from those with a 1-word `Alloc` would have a ~1.7x blowup in size for the whole data structure compared to a ZST `Alloc`. That seems pretty bad to me, and these kinds of applications are sort of the whole point of having `Alloc` be a trait.

---

### Comment 82 by glaebhoerl
_2017-10-16T21:40:32Z_

@cramertj 

> We could add something like this:

We're also going to have actual trait aliases :) https://github.com/rust-lang/rust/issues/41517

---

### Comment 83 by cramertj
_2017-10-17T00:05:02Z_

@glaebhoerl Yeah, but stabilization still seems a ways off since there isn't an implementation yet. If we disable manual impls of `Allocator` I think we can switch to trait-aliases backwards-compatibly when they arrive ;)

---

### Comment 84 by alexcrichton
_2017-10-17T20:29:40Z_

@joshlf 

> A 2x (or more) blowup in the space required to store such a common and fundamental type is concerning to me.

I'd imagine all implementations today are just a zero-size type or a pointer large, right? Isn't the possible optimization that some pointer-size-types can be zero sized? (or something like that?)

---

@cramertj 

> We could add something like this:

Indeed! We've then taken one trait to **three** though. In the past we've never had a great experience with such traits. For example `Box<Both>` does not cast to `Box<OnlyOneTrait>`. I'm sure we could wait for language features to smooth all this out, but it seems like those are a long way off, at best.

---

@SimonSapin 

> What still needs to be resolved to stabilize the whole thing?

I don't know. I wanted to start with the absolute smallest thing so there would be less debate. 

---

### Comment 85 by joshlf
_2017-10-17T20:33:18Z_

> I'd imagine all implementations today are just a zero-size type or a pointer large, right? Isn't the possible optimization that some pointer-size-types can be zero sized? (or something like that?)

Yeah, the idea is that, given a pointer to an object allocated from your your type of allocator, you can figure out which instance it came from (e.g., using inline metadata). Thus, the only information you need to deallocate is type information, not runtime information.

---

### Comment 86 by ruuda
_2017-10-17T20:39:28Z_

To come back to alignment on deallocate, I see two ways forward:

 * Stabilize as proposed (with alignment on deallocate). Giving away ownership of manually allocated memory would be impossible unless the `Layout` is included. In particular, it is impossible to build a `Vec` or `Box` or `String` or other `std` container with a stricter alignment than required (for instance because you don't want the boxed element to straddle a cache line), without deconstructing and deallocating it manually later (which is not always an option). Another example of something that would be impossible, is filling a `Vec` using simd operations, and then giving it away.

 * Do not require alignment on deallocate, and remove [the small-allocation optimization](https://github.com/rust-lang/rust/commit/45bf1ed1a1123122ded05ae2eedaf0f190e52726#diff-a7cb10d38695189cd9186f24c90e1634R199) from Windows’ `HeapAlloc`-based `alloc_system`. Always store the alignment. @alexcrichton, as you committed that code, do you remember why it was put there in the first place? Do we have any evidence that it saves a significant amount of memory for real-world applications? (With microbenchmarks it is possible to make the results come out either way depending the allocation size — unless `HeapAlloc` was rounding up sizes anyway.)

In any case, this is a very difficult trade off to make; the memory and performance impact will depend highly on the type of application, and which one to optimize for is application-specific as well.

---

### Comment 87 by joshlf
_2017-10-17T20:51:17Z_

I think we may actually be Just Fine (TM). Quoting the [`Alloc` docs](https://doc.rust-lang.org/nightly/alloc/allocator/trait.Alloc.html):

> Some of the methods require that a layout *fit* a memory block.
> What it means for a layout to "fit" a memory block means (or
> equivalently, for a memory block to "fit" a layout) is that the
> following two conditions must hold:
>
> 1. The block's starting address must be aligned to `layout.align()`.
>
> 2. The block's size must fall in the range `[use_min, use_max]`, where:
>
>    * `use_min` is `self.usable_size(layout).0`, and
>
>    * `use_max` is the capacity that was (or would have been)
>      returned when (if) the block was allocated via a call to
>      `alloc_excess` or `realloc_excess`.
>
> Note that:
>
>  * the size of the layout most recently used to allocate the block
>    is guaranteed to be in the range `[use_min, use_max]`, and
>
>  * a lower-bound on `use_max` can be safely approximated by a call to
>    `usable_size`.
>
>  * if a layout `k` fits a memory block (denoted by `ptr`)
>    currently allocated via an allocator `a`, then it is legal to
>    use that layout to deallocate it, i.e. `a.dealloc(ptr, k);`.

Note that last bullet. If I allocate with a layout with alignment `a`, then it should be legal for me to deallocate with alignment `b < a` because an object which is aligned to `a` is also aligned to `b`, and thus a layout with alignment `b` fits an object allocated with a layout with alignment `a` (and with the same size).

What this means is that you should be able to allocate with an alignment which is greater than the minimum alignment required for a particular type and then allow some other code to deallocate with the minimum alignment, and it should work.

---

### Comment 88 by glaebhoerl
_2017-10-17T23:19:57Z_

>  Isn't the possible optimization that some pointer-size-types can be zero sized? (or something like that?)

There was an RFC for this recently and it seems very unlikely that it could be done due to compatibility concerns: https://github.com/rust-lang/rfcs/pull/2040

> For example `Box<Both>` does not cast to `Box<OnlyOneTrait>`. I'm sure we could wait for language features to smooth all this out, but it seems like those are a long way off, at best.

Trait object upcasting on the other hand seems uncontroversially desirable, and mostly a question of effort / bandwidth / willpower to get it implemented. There was a thread recently: https://internals.rust-lang.org/t/trait-upcasting/5970

---

### Comment 89 by retep998
_2017-10-18T03:13:40Z_

@ruuda I was the one who wrote that `alloc_system` implementation originally. alexcrichton merely moved it around during the great allocator refactors of `<time period>`.

The current implementation *requires* that you deallocate with the same alignment specified that you allocated a given memory block with. Regardless of what the documentation may claim, this is the current reality that everyone must abide by until `alloc_system` on Windows is changed.

Allocations on Windows always use a multiple of `MEMORY_ALLOCATION_ALIGNMENT` (although they remember the size you allocated them with to the byte). `MEMORY_ALLOCATION_ALIGNMENT` is 8 on 32bit and 16 on 64bit. For overaligned types, because the alignment is greater than `MEMORY_ALLOCATION_ALIGNMENT`, the overhead caused by `alloc_system` is consistently the amount of alignment specified, so a 64byte aligned allocation would have 64 bytes of overhead.

If we decided to extend that overaligned trick to *all* allocations (which would get rid of the requirement to deallocate with the same alignment that you specified when allocating), then more allocations would have overhead. Allocations whose alignments are identical to `MEMORY_ALLOCATION_ALIGNMENT` will suffer a constant overhead of `MEMORY_ALLOCATION_ALIGNMENT` bytes. Allocations whose alignments are less than `MEMORY_ALLOCATION_ALIGNMENT` will suffer an overhead of `MEMORY_ALLOCATION_ALIGNMENT` bytes approximately *half* of the time. If the size of the allocation rounded up to `MEMORY_ALLOCATION_ALIGNMENT` is greater than or equal to the size of the allocation plus the size of a pointer, then there is no overhead, otherwise there is. Considering 99.99% of allocations will not be overaligned, do you really want to incur that sort of overhead on all those allocations?

---

### Comment 90 by alexcrichton
_2017-10-18T14:28:20Z_

@ruuda 

I personally feel that the implementation of `alloc_system` today on Windows is a bigger benefit than having the ability to relinquish ownership of an allocation to another container like `Vec`. AFAIK though there's no data to measure the impact of always padding with the alignment and not requiring an alignment on deallocation.

@joshlf 

I think that comment is wrong, as pointed out `alloc_system` on Windows relies on the same alignment being passed to deallocation as was passed on allocation.

---

### Comment 91 by ruuda
_2017-10-18T17:44:28Z_

> Considering 99.99% of allocations will not be overaligned, do you really want to incur that sort of overhead on all those allocations?

It depends on the application whether the overhead is significant, and whether to optimize for memory or performance. My suspicion is that for most applications either is fine, but a small minority cares deeply about memory, and they *really* cannot afford those extra bytes. And another small minority needs control over alignment, and they *really* need it.

---

### Comment 92 by joshlf
_2017-10-18T18:00:16Z_

@alexcrichton 
> I think that comment is wrong, as pointed out `alloc_system` on Windows relies on the same alignment being passed to deallocation as was passed on allocation.

Doesn't that imply that `alloc_system` on Windows doesn't actually properly implement the `Alloc` trait (and thus maybe we ought to change the requirements of the `Alloc` trait)?

----

@retep998 

If I'm reading your comment correctly, isn't that alignment overhead present for all allocations regardless of whether we need to be able to deallocate with a different alignment? That is, if I allocate 64 bytes with 64 byte alignment and I also deallocate with 64 byte alignment, the overhead you described is still present. Thus, it's not a feature of being able to deallocate with different alignments so much as it is a feature of requesting larger-than-normal alignments.

---

### Comment 93 by retep998
_2017-10-18T23:38:43Z_

@joshlf The overhead caused by `alloc_system` currently is due to requesting larger-than-normal alignments. If your alignment is less than or equal to `MEMORY_ALLOCATION_ALIGNMENT`, then there is no overhead caused by `alloc_system`.

However, if we changed the implementation to *allow* deallocating with different alignments, then the overhead would apply to *nearly all* allocations, regardless of alignment.

---

### Comment 94 by joshlf
_2017-10-18T23:46:55Z_

Ah I see; makes sense.

---

### Comment 95 by dtolnay
_2017-10-19T05:39:20Z_

What is the meaning of implementing Alloc for both Heap and &Heap? In what cases would the user use one of those impls vs the other?

Is this the first standard library API in which `*mut u8` would mean "pointer to whatever"? There is String::from_raw_parts but that one really does mean pointer to bytes. I am not a fan of `*mut u8` meaning "pointer to whatever" -- even C does better. What are some other options? Maybe a pointer to [opaque type](https://github.com/rust-lang/rust/pull/44295) would be more meaningful.

@rfcbot concern *mut u8

---

### Comment 96 by alexcrichton
_2017-10-19T14:01:19Z_

@dtolnay `Alloc for Heap` is sort of "standard" and `Alloc for &Heap` is like `Write for &T` where the trait requires `&mut self` but the implementation does not. Notably that means that types like `Heap` and `System` are threadsafe and do not need to be synchronized when allocating.

More importantly, though, usage of `#[global_allocator]` requires that the static it's attached to , which has type `T`, to have `Alloc for &T`. (aka all global allocators must be threadsafe)

For `*mut u8` I think that `*mut ()` may be interesting, but I don't personally feel too compelled to "get this right" here per se.

---

### Comment 97 by Amanieu
_2017-10-19T15:44:28Z_

The main advantage of `*mut u8` is that it is very convenient to use `.offset` with byte offsets.

---

### Comment 98 by joshlf
_2017-10-19T20:30:31Z_

> For `*mut u8` I think that `*mut ()` may be interesting, but I don't personally feel too compelled to "get this right" here per se.

If we go with `*mut u8` in a stable interface, aren't we locking ourselves in? In other words, once we stabilize this, we won't have a chance to "get this right" in the future.

Also, `*mut ()` seems a bit dangerous to me in case we ever make an optimization like [RFC 2040](https://github.com/rust-lang/rfcs/pull/2040) in the future.

---

### Comment 99 by joshlf
_2017-10-19T20:33:21Z_

> The main advantage of `*mut u8` is that it is very convenient to use .offset with byte offsets.

True, but you could easily do `let ptr = (foo as *mut u8)` and then go about your merry way. That doesn't seem like enough of a motivation to stick with `*mut u8` in the API if there are compelling alternatives (which, to be fair, I'm not sure there are).

---

### Comment 100 by cramertj
_2017-10-19T20:36:41Z_

> Also, *mut () seems a bit dangerous to me in case we ever make an optimization like RFC 2040 in the future.

That optimization will already probably never happen-- it'd break too much existing code. Even if it did, it would be applied to `&()` and `&mut ()`, not `*mut ()`.

---

### Comment 101 by joshlf
_2017-10-19T20:47:42Z_

If [RFC 1861](https://github.com/rust-lang/rfcs/blob/master/text/1861-extern-types.md) were close to being implemented/stabilized, I'd suggest using it:

```rust
extern { pub type void; }

pub unsafe trait Alloc {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut void, Error>;
    unsafe fn dealloc(&mut self, ptr: *mut void, layout: Layout);
    // ...
}
```

It's probably too far away though, right?

---

### Comment 102 by eddyb
_2017-10-20T11:50:03Z_

@joshlf I thought I saw a PR open about them, the remaining unknown is `DynSized`.

---

### Comment 103 by alkis
_2017-10-24T14:32:17Z_

Will this work for struct hack like objects? Say I have a `Node<T>` that looks like so:

```
struct Node<T> {
   size: u32,
   data: T,
   // followed by `size` bytes
}
```
and a value type:

```
struct V {
  a: u32,
  b: bool,
}
```

Now I want to allocate `Node<V>` with a string of size 7 in a *single* allocation. Ideally I want to make an allocation of size 16 align 4 and fit everything in it: 4 for `u32`, 5 for `V`, and 7 for the string bytes. This works because the last member of `V` has alignment 1 and the string bytes also have alignment 1.

Note that this is not allowed in C/C++ if the types are composed like above, as writing to packed storage is undefined behavior. I think this is a hole in the C/C++ standard that unfortunately cannot be fixed. I can expand on why this is broken but lets focus on Rust instead. Can this work? :-)



---

### Comment 104 by joshlf
_2017-10-24T15:19:55Z_

With respect to the size and alignment of the `Node<V>` structure itself, you're pretty much at the whim of the Rust compiler. It's UB (undefined behavior) to allocate with any size or alignment smaller than what Rust requires since Rust may make optimizations based on the assumption that any `Node<V>` object - on the stack, on the heap, behind a reference, etc - has a size and alignment matching what is expected at compile time.

In practice, it looks like the answer is unfortunately no: I ran [this program](https://play.rust-lang.org/?gist=4ae4593c269b3cc1d706f6a8f02d5d3e&version=stable) and found that, at least on the Rust Playground, `Node<V>` is given a size of 12 and an alignment of 4, meaning that any objects after the `Node<V>` must be offset by at least 12 bytes. It looks like the offset of the `data.b` field within the `Node<V>` is 8 bytes, implying that bytes 9-11 are trailing padding. Unfortunately, even though those padding bytes "aren't used" in some sense, the compiler still treats them as part of the `Node<V>`, and reserves the right to do anything it likes with them (most importantly, including writing to them when you assign to a `Node<V>`, implying that if you try to squirrel away extra data there, it may get overwritten).

(note, btw: you can't treat a type as packed that the Rust compiler doesn't think is packed. However, you _can_ tell the Rust compiler that something is packed, which will change the type's layout (removing padding), using [`repr(packed)`](https://doc.rust-lang.org/beta/nomicon/other-reprs.html#reprpacked))

However, with respect to laying out one object after another without having them both be part of the same Rust type, I'm almost 100% sure this is valid - after all, it's what `Vec` does. You can use the methods of the [`Layout` type](https://doc.rust-lang.org/nightly/alloc/allocator/struct.Layout.html) to dynamically calculate how much space is needed for the total allocation:

```rust
let node_layout = Layout::new::<Node<V>>();
// NOTE: This is only valid if the node_layout.align() is at least as large as mem::align_of_val("a")!
// NOTE: I'm assuming that the alignment of all strings is the same (since str is unsized, you can't do mem::align_of::<str>())
let padding = node_layout.padding_needed_for(mem::align_of_val("a"));
let total_size = node_layout.size() + padding + 7;
let total_layout = Layout::from_size_align(total_size, node_layout.align()).unwrap();
```

---

### Comment 105 by SimonSapin
_2017-10-24T15:35:01Z_

Would something like this work?

```rust
#[repr(C)]
struct Node<T> {
   size: u32,
   data: T,
   bytes: [u8; 0],
}
```

… then allocate with a larger size, and use `slice::from_raw_parts_mut(node.bytes.as_mut_ptr(), size)` ?

---

### Comment 106 by alkis
_2017-10-24T15:49:48Z_

Thanks @joshlf for the detailed answer! The TLDR for my usecase is that I can get a `Node<V>` of size 16 but only if V is `repr(packed)`. Otherwise the best I can do is size 19 (12 + 7).

@SimonSapin not sure; I will try.

---

### Comment 107 by Ericson2314
_2017-10-24T16:16:22Z_

Haven't really caught up with this thread, but I am *dead set* against stabilizing anything yet. We've not made progress implementing the the hard problems yet:

 1. Allocator-polymorphic collections
    - not even non-bloated box!
 2. Falliable collections

I think the design of the fundamental traits *will* affect the solutions of those: I've had little time for Rust for the past few months, but have argued this at times. I doubt I will have time to fully make my case here either, so I can only hope we first at least write up a *complete* solution to all of those: somebody prove me wrong that it's impossible to be rigorous (force correct usage), flexible, and ergonomic with the current traits. Or even just finish checking the boxes at the top.

---

### Comment 108 by joshlf
_2017-10-24T16:41:42Z_

Re: @Ericson2314's comment

I think that a relevant question related to the conflict between that perspective and @alexcrichton's desire to stabilize something is: How much benefit do we get from stabilizing a minimal interface? In particular, very few consumers will call `Alloc` methods directly (even most collections will probably use `Box` or some other similar container), so the real question becomes: what does stabilizing buy for users who will not be calling `Alloc` methods directly? Honestly the only serious use case I can think of is that it paves a path for allocator-polymorphic collections (which will likely be used by a much broader set of users), but it seems like that's blocked on #27336, which is far from being resolved. Maybe there are other large use cases I'm missing, but based on that quick analysis, I'm inclined to lean away from stabilization as having only marginal benefits at the cost of locking us into a design that we might later find to be suboptimal.

---

### Comment 109 by sfackler
_2017-10-24T16:42:55Z_

@joshlf it allows people to define and use their own global allocators.

---

### Comment 110 by joshlf
_2017-10-24T16:49:19Z_

Hmmm good point. would it be possible to stabilize specifying the global allocator without stabilizing `Alloc`? I.e., the code that implements `Alloc` would have to be unstable, but that would probably be encapsulated in its own crate, and the mechanism to mark that allocator as the global allocator would itself be stable. Or am I misunderstanding how stable/unstable and the stable compiler/nightly compiler interact?

---

### Comment 111 by Ericson2314
_2017-10-24T17:21:59Z_

Ah @joshlf remember that #27336 is a distraction, as per https://github.com/rust-lang/rust/issues/42774#issuecomment-317279035. I'm pretty sure we'll run into *other* problems---problems with the traits as they exist, which is why I want to work to commence work on that now. It's a lot easier to discuss those problems once they arrive for all to see than debate predicted futures post-#27336 .

---

### Comment 112 by sfackler
_2017-10-24T17:49:04Z_

@joshlf But you can't compile the crate that defines the global allocator with a stable compiler.

---

### Comment 113 by joshlf
_2017-10-24T18:18:52Z_

@sfackler Ah yes, there's that misunderstanding I was afraid of :P

---

### Comment 114 by gnzlbg
_2017-10-25T11:58:51Z_

I find the name `Excess(ptr, usize)` a bit confusing because the `usize` is not the `excess` in size of the requested allocation (as in the extra size allocated), but the `total` size of the allocation.

IMO `Total`, `Real`, `Usable`, or any name that conveys that the size is the total size or real size of the allocation is better than "excess", which I find misleading. The same applies for the `_excess` methods. 

---

### Comment 115 by arthurprs
_2017-10-25T14:22:39Z_

I agree with @gnzlbg above, I think a plain (ptr, usize) tuple would be just fine.

---

### Comment 116 by alexcrichton
_2017-10-25T14:49:48Z_

Note that `Excess` is not proposed to be stablized in the first pass, however

---

### Comment 117 by bstrie
_2017-10-25T21:47:17Z_

Posted this thread for discussion on reddit, which has some people with concerns: https://www.reddit.com/r/rust/comments/78dabn/custom_allocators_are_on_the_verge_of_being/

---

### Comment 118 by alexcrichton
_2017-10-31T22:22:28Z_

After further discussion with @rust-lang/libs today, I'd like to make a few tweaks to the stabilization proposal which can be summarized with:

* Add `alloc_zeroed` to the set of stabilized methods, otherwise having the same signature as `alloc`.
* Change `*mut u8` to `*mut void` in the API using `extern { type void; }` support, solving @dtolnay's concern and providing a way forward for unifying `c_void` across the ecosystem.
* Change the return type of `alloc` to `*mut void`, removing the `Result` and the `Error`

Perhaps the most contentious is the last point so I want to elaborate on that as well. This came out of discussion with the libs team today and specifically revolved around how (a) the `Result`-based interface has a less efficient ABI than a pointer-returning one and (b) almost no "production" allocators today provide the ability to learn anything more than "this just OOM'd". For performance we can mostly paper over it with inlining and such but it remains that the `Error` is an extra payload that's difficult to remove at the lowest layers.

The thinking for returning payloads of errors is that allocators can provide implementation-specific introspection to learn why an allocation failed and otherwise almost all consumers should only need to know whether the allocation succeeded or failed. Additionally this is intended to be a very low-level API which isn't actually called that often (instead, the typed APIs which wrap things up nicely should be called instead). In that sense it's not paramount that we have the most usable and ergonomic API for this location, but rather it's more important about enabling use cases without sacrificing performance.



---

### Comment 119 by dtolnay
_2017-10-31T23:20:28Z_

> The main advantage of `*mut u8` is that it is very convenient to use `.offset` with byte offsets.

In the libs meeting we also suggested `impl *mut void { fn offset }` which does not conflict with the existing `offset` defined for `T: Sized`. Could also be `byte_offset`.

---

### Comment 120 by joshlf
_2017-10-31T23:59:47Z_

+1 for using `*mut void` and `byte_offset`. Is there going to be an issue with stabilization of the extern types feature, or can we sidestep that issue because only the definition is unstable (and liballoc can do unstable things internally) and not the use (e.g., `let a: *mut void = ...` isn't unstable)?

---

### Comment 121 by sfackler
_2017-11-01T00:28:19Z_

Yep, we don't need to block on extern type stabilization. Even if extern type support gets deleted, the `void` we define for this can always be a magical type worst case.

---

### Comment 122 by cramertj
_2017-11-01T00:33:54Z_

Was there any further discussion in the libs meeting about whether or not `Alloc` and `Dealloc` should be separate traits?

---

### Comment 123 by sfackler
_2017-11-01T00:42:53Z_

We didn't touch on that specifically, but we generally felt that we shouldn't be diverting from prior art unless we have a particularly compelling reason to do so. In particular, C++'s Allocator concept does not have a similar split.

---

### Comment 124 by joshlf
_2017-11-01T00:46:06Z_

I'm not sure that's an apt comparison in this case. In C++, everything's explicitly freed, so there's no equivalent to `Box` that needs to store a copy of (or a reference to) its own allocator. That's what causes the large size blowup for us.

---

### Comment 125 by cramertj
_2017-11-01T00:48:25Z_

`std::unique_ptr` [is generic on a "Deleter"](http://en.cppreference.com/w/cpp/memory/unique_ptr).

---

### Comment 126 by sfackler
_2017-11-01T01:25:02Z_

@joshlf `unique_ptr` is the equivalent of `Box`, `vector` is the equivalent of `Vec`, `unordered_map` is the equivalent of `HashMap`, etc.

@cramertj Ah, interesting, I was only looking at collections types. It seems like that might be a thing to do then. We can always add it in later via blanket impls but it'd probably be cleaner to avoid that.

---

### Comment 127 by sfackler
_2017-11-01T02:23:04Z_

The blanket impl approach might be cleaner, actually:

```rust
pub trait Dealloc {
    fn dealloc(&self, ptr: *mut void, layout: Layout);
}

impl<T> Dealloc for T
where
    T: Alloc
{
    fn dealloc(&self, ptr: *mut void, layout: Layout) {
        <T as Alloc>::dealloc(self, ptr, layout)
    }
}
```

One less trait to worry about for the majority of use cases.

---

### Comment 128 by SimonSapin
_2017-11-01T07:03:37Z_

> * Change the return type of alloc to *mut void, removing the Result and the Error
>
> Perhaps the most contentious is the last point so I want to elaborate on that as well. This came out of discussion with the libs team today and specifically revolved around how (a) the Result-based interface has a less efficient ABI than a pointer-returning one and (b) almost no "production" allocators today provide the ability to learn anything more than "this just OOM'd". For performance we can mostly paper over it with inlining and such but it remains that the Error is an extra payload that's difficult to remove at the lowest layers.

I’m concerned that this would make it very easy to use the returned pointer without checking for null. It seems that the overhead could also be removed without adding this risk by returning `Result<NonZeroPtr<void>, AllocErr>` and making `AllocErr` zero-size?

(`NonZeroPtr` is a merged `ptr::Shared` and `ptr::Unique` as proposed in https://github.com/rust-lang/rust/issues/27730#issuecomment-316236397.)

---

### Comment 129 by alexcrichton
_2017-11-01T15:38:07Z_

@SimonSapin something like `Result<NonZeroPtr<void>, AllocErr>` requires *three* types to be stabilized, all of which are brand new and some of which have historically been languishing quite a bit for stabilization. Something like `void` isn't even required and is a nice-to-have (in my opinion).

I agree it's "easy to use it without checking for null", but this is, again, a very low-level API that's not intended to be heavily used, so I don't think we should optimize for caller ergonomics.

---

### Comment 130 by sfackler
_2017-11-01T15:42:01Z_

People can also build higher level abstractions like `alloc_one` on top of the low level `alloc` that could have more complex return types like `Result<NonZeroPtr<void>, AllocErr>`.

---

### Comment 131 by ruuda
_2017-11-01T20:06:55Z_

I agree that `AllocErr` would not be useful in practice, but how about just `Option<NonZeroPtr<void>>`? APIs that are impossible to accidentally misuse, without overhead, is one of the things that set Rust apart from C, and returning to C-style null pointers feels like a step backwards to me. Saying it is “a very low-level API that's not intended to be heavily used” is like saying that we should not care about memory safety on uncommon microcontroller architectures because they are very low-level and not heavily used.

---

### Comment 132 by sfackler
_2017-11-01T20:19:44Z_

Every interaction with the allocator involves unsafe code regardless of the return type of this function. Low level allocation APIs are possible to misuse whether the return type is `Option<NonZeroPtr<void>>` or `*mut void`.

`Alloc::alloc` **in particular** is the API that is low level and not intended to be heavily used. Methods like `Alloc::alloc_one<T>` or `Alloc::alloc_array<T>` are the alternatives that would be more heavily used and have a "nicer" return type.

A *stateful* `AllocError` is not worth it, but a zero sized type that implements Error and has a `Display` of `allocation failure` is nice to have. If we go the `NonZeroPtr<void>` route, I see `Result<NonZeroPtr<void>, AllocError>` as preferable to `Option<NonZeroPtr<void>>`.

---

### Comment 133 by Ericson2314
_2017-11-03T21:22:20Z_

Why the rush to stabilize :(!!  `Result<NonZeroPtr<void>, AllocErr>` is indisputably nicer for clients to use. Saying this is a "very low-level API" that need not be nice is just depressingly unambitious. Code at all levels should be as safe and maintainable as possible; obscure code that isn't constantly edited (and thus paged in people's short term memories) all the more so!

Furthermore, if we are to have have user-written allocate-polymorphic collections, which I certainly hope for, that's an open amount of fairly complex code using allocators directly.

---

### Comment 134 by Ericson2314
_2017-11-03T21:46:02Z_

Re dealloaction, operationally, we almost certainly want to reference/clone the alloactor just once per tree-based collection. That means passing in the allocator to each custom-allocator-box being destroyed. But, It's an open problem on how best to do this in Rust without linear types. Contrary to my previous comment, I'd be OK with some unsafe code in collections implementations for this, because the ideal usecase changes the implementation of `Box`, not the implementation of split allocator and deallocator traits. I.e. we can make stabilizable progress without blocking on linearity.

@sfackler I think we need some associated types connecting the deallocator to the allocator; it may be not possible to retrofit them.

---

### Comment 135 by sfackler
_2017-11-03T21:52:42Z_

@Ericson2314 There is a "rush" to stabilize because people want to use allocators to real things in the real world. This is not a science project.

What would that associated type be used for?

---

### Comment 136 by Ericson2314
_2017-11-03T22:02:45Z_

@sfackler people can still pin a nightly / and type of people who care about this sort of advance feature should be comfortable doing that. [If the issue is unstable rustc vs unstable Rust, that's a different problem that needs a policy fix.] Baking in lousy APIs, on the contrary, hamstrings us *forever*, unless we want to split the ecosystem with a new 2.0 std.

The associated types would relate the deallocator to the allocator. Each needs to know about that other for this to work. [There's still the issue of using the wrong (de)allocator of the right type, but I accept that no one has remotely proposed a solution to that.]

---

### Comment 137 by sfackler
_2017-11-03T22:11:22Z_

If people can just pin to a nightly, why do we have stable builds at all? The set of people who are directly interacting with allocator APIs is much smaller than the people who want to take advantage of those APIs by e.g. replacing the global allocator.

Can you write some code that shows why a deallocator needs to know the type of its associated allocator? Why doesn't C++'s allocator API need a similar mapping?

---

### Comment 138 by Ericson2314
_2017-11-03T22:43:17Z_

> If people can just pin to a nightly, why do we have stable builds at all?

To indicate language stability. Code you write against this version of things will never break. On a newer compiler. You pin a nightly when you need something so bad, it's not worth waiting for the final iteration of the feature deemed of quality worthy of that guarantee.

> The set of people who are directly interacting with allocator APIs is much smaller than the people who want to take advantage of those APIs by e.g. replacing the global allocator.

Aha! This would be for moving jemalloc out of tree, etc? No one has proposed stabilizing the awful hacks that allow choosing the global allocator, just the heap static itself? Or did I read the proposal wrong?

---

### Comment 139 by sfackler
_2017-11-03T22:48:31Z_

The awful hacks that allow for choosing the global allocator are proposed to be stabilized, which is half of what allows us to move jemalloc out of tree. This issue is the other half.

---

### Comment 140 by SimonSapin
_2017-11-03T22:48:57Z_

`#[global_allocator]` attribute stabilization: https://github.com/rust-lang/rust/issues/27389#issuecomment-336955367

---

### Comment 141 by Ericson2314
_2017-11-03T22:49:26Z_

Yikes

---

### Comment 142 by SimonSapin
_2017-11-04T07:18:16Z_

@Ericson2314 What do you think would be a non-awful way to select the global allocator?

---

### Comment 143 by Ericson2314
_2017-11-06T20:44:02Z_

(Responded in https://github.com/rust-lang/rust/issues/27389#issuecomment-342285805)

---

### Comment 144 by dtolnay
_2017-11-07T21:08:33Z_

The proposal has been amended to use *mut void.

@rfcbot resolved *mut u8

---

### Comment 145 by cramertj
_2017-11-07T21:21:00Z_

@rfcbot reviewed

After some discussion on IRC, I'm approving this with the understanding that we _do not_ intend to stabilize a `Box` generic on `Alloc`, but instead on some `Dealloc` trait with an appropriate blanket impl, as suggested by @sfackler [here](https://github.com/rust-lang/rust/issues/32838#issuecomment-340959804). Please let me know if I've misunderstood the intention.

---

### Comment 146 by joshlf
_2017-11-07T21:30:00Z_

@cramertj Just to clarify, it's possible to add that blanket impl after the fact and not break the `Alloc` definition that we're stabilizing here?

---

### Comment 147 by sfackler
_2017-11-07T21:36:35Z_

@joshlf yep, it'd look like this: https://github.com/rust-lang/rust/issues/32838#issuecomment-340959804

---

### Comment 148 by joshlf
_2017-11-07T21:41:32Z_

How will we specify the `Dealloc` for a given `Alloc`? I'd imagine something like this?

```rust
pub unsafe trait Alloc {
    type Dealloc: Dealloc = Self;
    ...
}
```

---

### Comment 149 by cramertj
_2017-11-07T21:43:26Z_

I guess that puts us in thorny territory WRT https://github.com/rust-lang/rust/issues/29661.

---

### Comment 150 by joshlf
_2017-11-07T21:44:59Z_

Yeah, I don't think there's a way to have the addition of `Dealloc` be backwards-compatible with existing definitions of `Alloc` (which don't have that associated type) without having a default.

---

### Comment 151 by sfackler
_2017-11-07T22:02:40Z_

If you wanted to automatically be able to grab the deallocator corresponding to an allocator, you'd need more than just an associated type, but a function to produce a deallocator value.

But, this can be handled in the future with that stuff being attached to a separate subtrait of `Alloc` I think.

---

### Comment 152 by cramertj
_2017-11-07T22:05:46Z_

@sfackler i'm not sure I understand. Can you write out the signature of `Box::new` under your design?

---

### Comment 153 by sfackler
_2017-11-07T22:43:24Z_

This is ignoring placement syntax and all of that, but one way you could do it would be

```rust
pub struct Box<T, D>(NonZeroPtr<T>, D);

impl<T, D> Box<T, D>
where
    D: Dealloc
{
    fn new<A>(alloc: A, value: T) -> Box<T, D>
    where
        A: Alloc<Dealloc = D>
    {
        let ptr = alloc.alloc_one().unwrap_or_else(|_| alloc.oom());
        ptr::write(&value, ptr);
        let deallocator = alloc.deallocator();
        Box(ptr, deallocator)
    }
}
```

Notably, we need to actually be able to produce an instance of the deallocator, not just know its type. You could also parameterize the `Box` over the `Alloc` and store `A::Dealloc` instead, which might help with type inference. We can make this work after this stabilization by moving `Dealloc` and `deallocator` to a separate trait:

```rust
pub trait SplitAlloc: Alloc {
    type Dealloc;

    fn deallocator(&self) -> Self::Dealloc;
}
```

---

### Comment 154 by joshlf
_2017-11-07T23:00:01Z_

But what would the impl of `Drop` look like?

---

### Comment 155 by sfackler
_2017-11-07T23:30:57Z_

```rust
impl<T, D> Drop for Box<T, D>
where
    D: Dealloc
{
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.0);
            self.1.dealloc_one(self.0);
        }
    }
}
```

---

### Comment 156 by joshlf
_2017-11-07T23:36:41Z_

But assuming we stabilize `Alloc` first, then not all `Alloc`s will implement `Dealloc`, right? And I thought impl specialization was still a ways off? In other words, in theory, you'd want to do something like the following, but I don't think it works yet?

```rust
impl<T, D> Drop for Box<T, D> where D: Dealloc { ... }
impl<T, A> Drop for Box<T, A> where A: Alloc { ... }
```

---

### Comment 157 by sfackler
_2017-11-07T23:40:22Z_

If anything, we'd have a

```rust
default impl<T> SplitAlloc for T
where
    T: Alloc { ... }
```

But I don't think that'd really be necessary. The use cases for custom allocators and global allocators are distinct enough that I wouldn't assume there'd be a ton of overlap between them.

---

### Comment 158 by joshlf
_2017-11-07T23:49:26Z_

I suppose that could work. It seems much cleaner to me, though, to just have `Dealloc` right off the bat so we can have the simpler interface. I imagine we could have a pretty simple, uncontroversial interface that would require no change to existing code that already implements `Alloc`:

```rust
unsafe trait Dealloc {
    fn dealloc(&mut self, ptr: *mut void, layout: Layout);
}

impl<T> Dealloc for T
where
    T: Alloc
{
    fn dealloc(&self, ptr: *mut void, layout: Layout) {
        <T as Alloc>::dealloc(self, ptr, layout)
    }
}

unsafe trait Alloc {
    type Dealloc: Dealloc = &mut Self;
    fn deallocator(&mut self) -> Self::Dealloc { self }
    ...
}
```

---

### Comment 159 by sfackler
_2017-11-07T23:52:03Z_

I though associated type defaults were problematic?

---

### Comment 160 by sfackler
_2017-11-07T23:54:11Z_

A `Dealloc` that's a mutable reference to the allocator seems not all that useful - you can only allocate one thing at a time, right?

---

### Comment 161 by joshlf
_2017-11-07T23:56:26Z_

> I though associated type defaults were problematic?

Oh I guess associated type defaults are far enough away that we can't rely on them.

Still, we could have the simpler:

```rust
unsafe trait Dealloc {
    fn dealloc(&mut self, ptr: *mut void, layout: Layout);
}

impl<T> Dealloc for T
where
    T: Alloc
{
    fn dealloc(&self, ptr: *mut void, layout: Layout) {
        <T as Alloc>::dealloc(self, ptr, layout)
    }
}

unsafe trait Alloc {
    type Dealloc: Dealloc;
    fn deallocator(&mut self) -> Self::Dealloc;
    ...
}
```

and just require the implementor to write a bit of boilerplate.

> A `Dealloc` that's a mutable reference to the allocator seems not all that useful - you can only allocate one thing at a time, right?

Yeah, good point. Probably a moot point anyway given your other comment.

---

### Comment 162 by sfackler
_2017-11-07T23:59:44Z_

Should `deallocator` take `self`, `&self`, or `&mut self`?

---

### Comment 163 by joshlf
_2017-11-08T00:03:04Z_

Probably `&mut self` to be consistent with the other methods.

---

### Comment 164 by sfackler
_2017-11-08T00:07:23Z_

Are there any allocators that would prefer to take self by value so they don't have to e.g. clone state?

---

### Comment 165 by joshlf
_2017-11-08T00:08:13Z_

The problem with taking `self` by value is that it precludes getting a `Dealloc` and then continuing to allocate.

---

### Comment 166 by sfackler
_2017-11-08T00:11:33Z_

I'm thinking of a hypothetical "oneshot" allocator, though I don't know how much of a real thing that is.

---

### Comment 167 by joshlf
_2017-11-08T00:16:55Z_

Such an allocator might exist, but taking `self` by value would require that _all_ allocators work that way, and would preclude any allocators allowing allocation after `deallocator` has been called.

---

### Comment 168 by sfackler
_2017-11-08T00:19:15Z_

I would still like to see some of this implemented and used in collections before we think about stabilizing it.

---

### Comment 169 by joshlf
_2017-11-08T01:27:14Z_

Do you think https://github.com/rust-lang/rust/issues/27336 or the points discussed in https://github.com/rust-lang/rust/issues/32838#issuecomment-339066870 will allow us to move forward on collections?

---

### Comment 170 by sfackler
_2017-11-08T01:35:06Z_

I'm worried about the type alias approach's impact on documentation readability. A (very verbose) way to allow progress would be to wrap types:

```rust
pub struct Vec<T>(alloc::Vec<T, Heap>);

impl<T> Vec<T> {
    // forwarding impls for everything
}
```

---

### Comment 171 by cramertj
_2017-11-08T01:36:44Z_

I know it's a pain, but it seems like the changes we're discussing here are big enough that if we decide to go forward with split alloc/dealloc traits, we should try them out in std first and then re-FCP.

---

### Comment 172 by sfackler
_2017-11-08T01:41:33Z_

What is the timeline on waiting on this stuff to get implemented?

---

### Comment 173 by gnzlbg
_2017-11-10T09:33:21Z_

The `grow_in_place` method doesn't return any kind of excess capacity. It currently calls `usable_size` with a layout, extends the allocation to _at least_ fit this layout, but if the allocation is extended beyond that layout users have no way to know.

---

### Comment 174 by gnzlbg
_2017-11-10T09:42:26Z_

I am having a hard time understanding the advantage of the `alloc` and `realloc` methods over `alloc_excess` and `realloc_excess`. 

An allocator needs to find a suitable memory block to perform an allocation: this requires knowing the size of the memory block. Whether the allocator then returns a pointer, or the tuple "pointer and size of the memory block" does not make any measurable performance differences. 

So `alloc` and `realloc` just increase the API surface and seem to encourage writing less performant code. Why do we have them in the API at all? What's their advantage?

---

EDIT: or in other words: all potentially allocating functions in the API should return `Excess`, which basically removes the need for all the `_excess` methods.

---

### Comment 175 by sfackler
_2017-11-10T18:26:24Z_

Excess is only interesting for use cases that involve arrays which can grow. It's not useful or relevant for `Box` or `BTreeMap`, for example. There may be some cost in computing what the excess is, and there's certainly a more complex API, so it doesn't seem to me like code that doesn't care about excess capacity should be forced to pay for it.

---

### Comment 176 by gnzlbg
_2017-11-13T13:02:25Z_

> There may be some cost in computing what the excess is

Can you give an example? I don't know, and cannot imagine, an allocator that is able to allocate memory but that does not know how much memory it is actually allocating (which is what `Excess` is: real amount of memory allocated; we should rename it).

The only commonly used `Alloc`ator where this might be slightly controversial is POSIX `malloc`, which even though it always computes the `Excess` internally, does not expose it as part of its C API. However, returning the requested size as the `Excess` is ok, portable, simple, incurs no cost at all, and is what everybody using POSIX `malloc` is already assuming anyways. 

`jemalloc` and basically any other `Alloc`ator out there provide API's that returns the `Excess` without incurring any costs, so for those allocators, returning the `Excess` is zero cost as well. 

>  There may be some cost in computing what the excess is, and there's certainly a more complex API, so it doesn't seem to me like code that doesn't care about excess capacity should be forced to pay for it.

Right now everybody is already paying the price of the allocator trait having two APIs for allocating memory. And while one can build an `Excess`-less API on top of an `Excess`-full one`, the opposite is not true. So I wonder why it isn't done like this:
- `Alloc` trait methods always return `Excess`
- add an `ExcessLessAlloc` trait that just drops off the `Excess` from `Alloc` methods for all users that 1) care enough to use `Alloc` but 2) don't care about the real amount of memory currently being allocated (looks like a niche to me, but I still think that such an API is nice to have)
- if one day somebody discovers a way to implement `Alloc`ators with fast-paths for `Excess`-less methods, we can always provide a custom implementation of `ExcessLessAlloc` for it. 

FWIW I just landed on this thread again because I can't implement what I want on top of `Alloc`. I mentioned that it is missing `grow_in_place_excess` before,  but I just got stuck again because it is also missing `alloc_zeroed_excess` (and who knows what else).

I'd be more comfortable if the stabilization here would focus on stabilizing an `Excess`-full API first. Even if its API is not the most ergonomic for all uses, such an API would at least allow all uses which is a necessary condition to show that the design is not flawed.

---

### Comment 177 by joshlf
_2017-11-14T21:12:57Z_

> Can you give an example? I don't know, and cannot imagine, an allocator that is able to allocate memory but that does not know how much memory it is actually allocating (which is what `Excess` is: real amount of memory allocated; we should rename it).

Most allocators today use size classes, where each size class allocates only objects of a particular fixed size, and allocation requests that don't fit a particular size class are rounded up to the smallest size class that they fit inside. In this scheme, it's common to do things like having an array of size class objects and then doing `classes[size / SIZE_QUANTUM].alloc()`. In that world, figuring out what size class is used takes extra instructions: e.g., `let excess = classes[size / SIZE_QUANTUM].size`. It may not be a lot, but the performance of high-performance allocators (like jemalloc) are measured in single clock cycles, so it could represent meaningful overhead, especially if that size ends up getting passed through a chain of function returns.

---

### Comment 178 by sfackler
_2017-11-14T21:17:30Z_

> Can you give an example?

At least going off of your PR to alloc_jemalloc, `alloc_excess` is pretty clearly running more code than `alloc`: https://github.com/rust-lang/rust/pull/45514/files.

---

### Comment 179 by gnzlbg
_2017-11-15T19:16:53Z_

> In this scheme, it's common to do things like having an array of size class objects and then doing classes[size / SIZE_QUANTUM].alloc(). In that world, figuring out what size class is used takes extra instructions: e.g., let excess = classes[size / SIZE_QUANTUM].size

So let me see if I follow properly:

```rust
// This happens in both cases:
let size_class = classes[size / SIZE_QUANTUM];
let ptr = size_class.alloc(); 
// This would happen only if you need to return the size:
let size = size_class.size;
return (ptr, size);
```

Is that it? 


--- 

> At least going off of your PR to alloc_jemalloc, alloc_excess is pretty clearly running more code than alloc

That PR was a bugfix (not a perf fix), there are many things wrong with the current state of our jemalloc layer perf-wise but since that PR it at least returns what it should:

- `nallocx` is a `const` function in the GCC sense, that is, a true pure function. This means it has no side-effects, its results depends on its arguments only, it accesses no global state, its arguments are not pointers (so the function cannot access global state throw them), and for C/C++ programs LLVM can use this information to elide the call if the result is not used. AFAIK Rust currently cannot mark FFI C functions as `const fn` or similar. So this is the first thing that could be fixed and that would make `realloc_excess` zero-cost for those that don't use the excess as long as inlining and optimizations work properly.
- `nallocx` is always computed for aligned allocations inside `mallocx`, that is, all code is already comptuing it, but `mallocx` throws its result away, so here we are actually computing it twice, and in some cases `nallocx` is almost as expensive as `mallocx`... I have a [fork](https://github.com/gnzlbg/jemallocator) of jemallocator that has some benchmarks for things like this in its branches, but this must be fixed upstream by jemalloc by providing an API that does not throw this away. This fix, however, only affects those that are currently using the `Excess`.
- and then is the issue that we are computing the align flags twice, but that is something that LLVM can optimize on our side (and trivial to fix).

So yes, it looks like more code, but this extra code is code that we are actually calling twice, because the first time that we called it we threw the results away. It is not impossible to fix, but I haven't found the time to do this yet.

---

EDIT: @sfackler I managed to free some time for it today and was able to make  `alloc_excess` "free" with respect to `alloc` in jemallocs slow path, and have only an overhead of ~1ns in jemallocs' fast path. I haven't really looked at the fast path in much detail, but it might be possible to improve this further. The details are here: https://github.com/jemalloc/jemalloc/issues/1074#issuecomment-345040339


---

### Comment 180 by joshlf
_2017-11-15T19:44:23Z_

> Is that it?

Yes.

---

### Comment 181 by sfackler
_2017-11-21T03:55:14Z_

> So this is the first thing that could be fixed and that would make realloc_excess zero-cost for those that don't use the excess as long as inlining and optimizations work properly.

When used as the global allocator, none of this can be inlined.

> Even if its API is not the most ergonomic for all uses, such an API would at least allow all uses which is a necessary condition to show that the design is not flawed.

There is literally zero code on Github that calls `alloc_excess`. If this is such a crucially important feature, why has no one ever used it? C++'s allocation APIs do not provide access to excess capacity. It seems incredibly straightforward to add/stabilize these features in the future in a backwards compatible way if there is actual concrete evidence that they improve performance and anyone actually cares enough to use them.

---

### Comment 182 by gnzlbg
_2017-11-21T09:29:31Z_

> When used as the global allocator, none of this can be inlined.

Then that is a problem that we should try to solve, at least for LTO builds, because global allocators like `jemalloc` rely on this: `nallocx` is the way it is _by design_, and the first recommendation jemalloc's devs made us regarding `alloc_excess` performance is that we should have those calls inlined, and we should propagate C attributes properly, so that the compiler removes the `nallocx` calls from the call-sites that do not use the `Excess`, like C and C++ compilers do. 

Even if we can't do that, the `Excess` API can still be made zero-cost by patching the `jemalloc` API (I have an initial implementation of such a patch in my rust-lang/jemalloc fork). We could either maintain that API ourselves, or try to land it upstream, but for that to land upstream we must make a good case about why these other languages can perform these optimizations and Rust cannot. Or we must have another argument, like this new API is significantly faster than `mallocx + nallocx` for those users that do need the `Excess`. 

> If this is such a crucially important feature, why has no one ever used it?

That's a good question.  `std::Vec` is the poster-child for using the `Excess` API, but it currently does not use it, and all my previous comments stating "this and that are missing from the `Excess` API" were me trying to make `Vec` use it. The `Excess` API:

- was not returning the `Excess` at all: https://github.com/rust-lang/rust/pull/45514
- is missing functionality like `grow_in_place_excess` and `alloc_zeroed_excess`, 
- cannot propagate C attributes in FFI properly: https://github.com/rust-lang/rust/issues/46046
- ... (I doubt it ends here)

I cannot know why nobody is using this API. But given that not even the `std` library can use it for the data-structure it is best suited for (`Vec`), if I had to guess, I would say that the main reason is that this API is currently broken. 

If I had to guess even further, I would say that not even those who designed this API have used it, mainly because no single `std` collection uses it (which is where I expect this API to be tested at first), and also because using `_excess` and `Excess` everywhere to mean `usable_size`/`allocation_size` is extremely confusing/annoying to program with. 

This is probably because more work was put into the `Excess`-less APIs, and when you have two APIs, it is hard to keep them in sync, it is hard for users to discover both and know which to use, and finally, it is hard for users to prefer convenience over doing the right thing. 

---

### Comment 183 by gnzlbg
_2017-11-21T09:34:32Z_

Or in other words, if I have two competing APIs, and I put 100% of the work into improving one, and 0% of the work into improving the other, it isn't surprising to reach the conclusion that one is in practice significantly better than the other.

---

### Comment 184 by sfackler
_2017-11-21T18:16:15Z_

As far as I can tell, these are the only two calls to `nallocx` outside of jemalloc tests on Github:

https://github.com/facebook/folly/blob/f2925b23df8d85ebca72d62a69f1282528c086de/folly/detail/ThreadLocalDetail.cpp#L182
https://github.com/louishust/mysql5.6.14_tokudb/blob/4897660dee3e8e340a1e6c8c597f3b2b7420654a/storage/tokudb/ft-index/ftcxx/malloc_utils.hpp#L91

Neither of them resemble the current `alloc_excess` API, but are rather used standalone to compute an allocation size before it's made.

Apache Arrow looked into using `nallocx` in their implementation but found things did not work out well:

https://issues.apache.org/jira/browse/ARROW-464

These are basically the only references to `nallocx` I can find. Why is it important that the initial implementation of allocator APIs support such an obscure feature?

---

### Comment 185 by gnzlbg
_2017-11-21T22:14:53Z_

> As far as I can tell, these are the only two calls to nallocx outside of jemalloc tests on Github:

From the top of my head I know that at least facebook's vector type is using it via facebook's malloc implementation ([malloc](https://github.com/facebook/folly/blob/a15fcb1e76444f7d464b263ad37bf3b5fbfdf33e/folly/memory/Malloc.h#L208) and [fbvector growth policy](https://github.com/facebook/folly/blob/master/folly/FBVector.h#L1243); that is a big chunk of C++'s vectors at facebook use this) and also that Chapel used it to improve the performance of their `String` type ([here](https://github.com/chapel-lang/chapel/blob/2bb46054899096709fd847653d5d6274b22e29bb/runtime/include/mem/jemalloc/chpl-mem-impl.h#L81) and the [tracking issue](https://github.com/chapel-lang/chapel/issues/5272)). So maybe today wasn't Github's best day?

> Why is it important that the initial implementation of allocator APIs support such an obscure feature?

The initial implementation of an allocator API does not need to support this feature.

But good support for this feature should block the stabilization of such an API.

---

### Comment 186 by hanna-kruppe
_2017-11-21T22:17:27Z_

Why should it block stabilization if it can be added backwards-compatibly later?

---

### Comment 187 by gnzlbg
_2017-11-21T22:19:31Z_

> Why should it block stabilization if it can be added backwards-compatibly later?

Because for me at least it means that only half of the design space has been sufficiently explored.

---

### Comment 188 by hanna-kruppe
_2017-11-21T22:22:08Z_

Do you expect the non-excess related portions of the API will be affected by the design of the excess-related functionality? I admit I've only followed that discussion half-heartedly but it seems unlikely to me.

---

### Comment 189 by gnzlbg
_2017-11-21T22:39:34Z_

If we can't make this API: 

```rust
fn alloc(...) -> (*mut u8, usize) { 
   // worst case system API:
   let ptr = malloc(...);
   let excess = malloc_excess(...);
   (ptr, excess)
}
let (ptr, _) = alloc(...); // drop the excess
```
as efficient as this one:

```rust
fn alloc(...) -> *mut u8 { 
   // worst case system API:
   malloc(...)
}
let ptr = alloc(...);
```

then we have bigger problems. 

> Do you expect the non-excess related portions of the API will be affected by the design of the excess-related functionality?

So yes, I expect a good excess-API to have a huge effect on the design of the non-excess related functionality: it would completely remove it. 

That would prevent the current situation of having two APIs that are out of sync, and in which the excess-api has less functionality than the excess-less one. While one can build an excess-less API on top of an excess-full one, the opposite is not true. 

Those who want to drop the `Excess` should just drop it.

---

### Comment 190 by joshlf
_2017-11-21T22:43:39Z_

To clarify, if there were some way of adding an `alloc_excess` method after the fact in a backwards-compatible way, then you'd be OK with it? (but of course, stabilizing without `alloc_excess` means that adding it later would be a breaking change; I'm just asking so I understand your reasoning)

---

### Comment 191 by sfackler
_2017-11-21T22:48:19Z_

@joshlf It is very straightforward to do that.

---

### Comment 192 by rfcbot
_2017-11-21T22:48:21Z_

:bell: **This is now entering its final comment period**, as per the [review above](https://github.com/rust-lang/rust/issues/32838#issuecomment-336980230). :bell:

---

### Comment 193 by sfackler
_2017-11-21T22:49:37Z_

> Those who want to drop the Excess should just drop it.

Alternatively, the 0.01% of people that care about excess capacity can use another method.

---

### Comment 194 by joshlf
_2017-11-21T23:04:40Z_

@sfackler This is what I get for taking a two-week break from rust - I forget about default method impls :)

---

### Comment 195 by gnzlbg
_2017-11-21T23:34:25Z_

> Alternatively, the 0.01% of people that care about excess capacity can use another method.

Where you are getting this number?

All of my Rust data-structures are flat in memory. The ability to do that is the only reason I use Rust; if I could just Box everything I would be using a different language. So I don't care about the `Excess` the `0.01%` of the time, I care about it all the time. 

I understand that this is domain specific, and that in other domains people would never care about the `Excess`, but I doubt that only 0.01% of Rust users care about this (I mean, a lot of people use `Vec` and `String`, which are the poster-child data-structures for `Excess`). 

---

### Comment 196 by sfackler
_2017-11-21T23:49:53Z_

I am getting that number from the fact that there are approx 4 things in total that use nallocx, compared to the set of things that use malloc.

---

### Comment 197 by hanna-kruppe
_2017-11-22T00:12:51Z_

@gnzlbg 

Are you suggesting that if we did it "right" from the start, we'd have just `fn alloc(layout) -> (ptr, excess)` and no `fn alloc(layout) -> ptr` at all? That seems far from obvious to me. Even if excess is available, it seems natural to have the latter API for the use cases where excess doesn't matter (e.g., most tree structures), even if it's implemented as `alloc_excess(layout).0`.

---

### Comment 198 by gnzlbg
_2017-11-22T10:40:44Z_

@rkruppe 

> That seems far from obvious to me. Even if excess is available, it seems natural to have the latter API for the use cases where excess doesn't matter (e.g., most tree structures), even if it's implemented as alloc_excess(layout).0.

Currently, the excess-full API is implemented on top of the excess-less one. Implementing `Alloc` for an excess-less allocator requires the user to provide the `alloc` and `dealloc` methods. 

However, if I want to implement `Alloc` for an excess-full allocator, I need to provide more methods (at least `alloc_excess`, but this grows if we go into `realloc_excess`, `alloc_zeroed_excess`, `grow_in_place_excess`, ...).

If we were to do it the other way around, that is, implement the excess-less API as a nicety on top of the excess-full one, then implementing `alloc_excess` and `dealloc` suffices for supporting both types of allocators. 

The users that don't care or can't return or query the excess can just return the input size or layout (which is a tiny inconvenience), but the users that can handle and want to handle the excess don't need to implement any more methods.

--- 

@sfackler 

> I am getting that number from the fact that there are approx 4 things in total that use nallocx, compared to the set of things that use malloc.

Given these facts about `_excess` usage in the Rust ecosystem:

- 0 things in total use `_excess` in the rust ecosystem
- 0 things in total use `_excess` in the rust std library
- not even `Vec` and `String` can use the `_excess` API properly in the rust `std` library
- the `_excess` API is unstable, out-of-sync with the excess-less API, buggy until very recently (did not even return the `excess` at all), ...

 and given these facts about the usage of `_excess` in other languages: 

- jemalloc's API is not natively supported by C or C++ programs due to backwards compatibility
- C and C++ programs that want to use jemalloc's excess API need to go way out of their way to use it by: 
    - opting out of the system allocator and into jemalloc (or tcmalloc)
    - re-implement their language's std library (in the case of C++, implement an incompatible std library)
    - write their whole stack on top of this incompatible std library
- some communities (firefox uses it, facebook reimplements the collections in the C++ standard library to be able to use it, ...) still go out of their way to use it.

These two arguments look plausible to me:

- The `excess` API in `std` is not usable, therefore the `std` library cannot use it, therefore nobody can, which is why it isn't used even once in the Rust ecosystem.
- Even though C and C++ make it close to impossible to use this API, big projects with manpower go to great lengths to use it, therefore at least some potentially tiny community of people care _a lot_ about it.

Your argument:

 - Nobody uses the `_excess` API, therefore only 0.01% of the people care about it. 

does not. 

---

### Comment 199 by pnkfelix
_2017-11-22T11:59:30Z_

@alexcrichton The decision to switch from `-> Result<*mut u8, AllocErr>` to `-> *mut void` may come as a significant surprise to people who followed the original development of the allocator RFC's. 

I don't disagree with [the points you make](https://github.com/rust-lang/rust/issues/32838#issuecomment-336957415), but it nonetheless seemed like a fair number of people would have been willing to live with the "heavy-weightness" of `Result` over the increased likelihood of missing a null-check on the returned value.
 * I am ignoring the runtime efficiency issues imposed by the ABI because I, like @alexcrichton, assume that we could deal with those in some way via compiler tricks.

Is there some way we could get increased visibility on *that* late change on its own?

One way (off the top of my head): *Change* the signature now, in a PR on its own, on the master branch, while `Allocator` is still unstable. And then see who complains on the PR (and who celebrates!).
 * Is this too heavy-handed? Seems like it is by definitions less heavy-handed than coupling such a change with stabilization...

---

### Comment 200 by pnkfelix
_2017-11-22T12:05:33Z_

On the subject of whether to return `*mut void` or to return `Result<*mut void, AllocErr>`: Its possible that we should be revisiting the idea of separate "high-level" and "low-level" allocator traits, as discussed in [take II of the Allocator RFC](https://github.com/rust-lang/rfcs/pull/244).

---

### Comment 201 by pnkfelix
_2017-11-22T12:13:08Z_

(Obviously if I had a serious objection to the `*mut void` return value, then I would file it as a concern via the fcpbot. But at this point I'm pretty much trusting the judgement of the libs team, perhaps in some part due to fatigue over this allocator saga.)

---

### Comment 202 by joshlf
_2017-11-22T15:08:25Z_

@pnkfelix 
> The decision to switch from `-> Result<*mut u8, AllocErr>` to `-> *mut void` may come as a significant surprise to people who followed the original development of the allocator RFC's.

The latter implies that, as discussed, the only error we care to express is OOM. Thus, a slightly lighter-weight in-between that still has the benefit of protection against accidental failure to check for errors is `-> Option<*mut void>`.

---

### Comment 203 by sfackler
_2017-11-22T16:20:32Z_

@gnzlbg 

> The excess API in std is not usable, therefore the std library cannot use it, therefore nobody can, which is why it isn't used even once in the Rust ecosystem.

Then go fix it.

@pnkfelix 

> On the subject of whether to return *mut void or to return Result<*mut void, AllocErr>: Its possible that we should be revisiting the idea of separate "high-level" and "low-level" allocator traits, as discussed in take II of the Allocator RFC.

Those were basically our thoughts, except that the high level API would be in `Alloc` itself as `alloc_one`, `alloc_array` etc. We can even let those develop in the ecosystem first as extension traits to see what APIs people converge on.

---

### Comment 204 by gnzlbg
_2017-11-25T09:57:55Z_

@pnkfelix 

> The reason I made Layout only implement Clone and not Copy is that I wanted to leave open the possibility of adding more structure to the Layout type. In particular, I still am interested in trying to have the Layout attempt to track any type structure used to construct it (e.g. 16-array of struct { x: u8, y: [char; 215] }), so that allocators would have the option of exposing instrumentation routines that report on what types their current contents are composes from.

Has this been experimented with somewhere?

---

### Comment 205 by gnzlbg
_2017-11-29T09:56:12Z_

@sfackler I have done most of it already, and all of it can be done with the duplicated API (no excess + `_excess` methods). I would be fine with having two APIs and not having a complete `_excess` API right now.

The only thing that still worries me a bit is that to implement an allocator right now one needs to implement `alloc + dealloc`, but `alloc_excess + dealloc` should also work as well. Would it be possible to give `alloc` a default implementation in terms of `alloc_excess` later or is that a not possible or a breaking change? In practice, most allocators are going to implement most methods anyways, so this is not a big deal, but more like a wish. 

---

 `jemallocator` implements `Alloc` twice (for `Jemalloc` and `&Jemalloc`), where the `Jemalloc` implementation for some `method` is just a `(&*self).method(...)` that forwards the method call to the `&Jemalloc` implementation. This means that one must manually keep both implementations of `Alloc` for `Jemalloc` in sync. Whether getting different behaviors for the `&/_` implementations can be tragic or not, I don't know. 

---

I've found it very hard to find out what people are actually doing with the `Alloc` trait in practice. The only projects that I've found that are using it are going to stay using nightly anyways (servo, redox), and are only using it to change the global allocator. It worries me a lot that I could not find any project that uses it as a collection type parameter (maybe I was just unlucky and there are some?). I was particularly looking for examples of implementing `SmallVec` and `ArrayVec` on top of a `Vec`-like type (since `std::Vec` doesn't have an `Alloc` type parameter yet), and also was wondering how cloning between these types (`Vec`s with a different `Alloc`ator) would work (the same probably applies to cloning `Box`es with different `Alloc`s). Are there examples of how these implementations would look like somewhere?

---

### Comment 206 by SimonSapin
_2017-11-29T10:04:21Z_

> The only projects that I've found that are using it are going to stay using nightly anyways (servo, redox)

For what it’s worth, Servo is trying to move off of unstable features where possible: https://github.com/servo/servo/issues/5286

This is also a chicken-and-egg problem. Many projects don’t use `Alloc` yet *because* it’s still unstable.

---

### Comment 207 by sfackler
_2017-11-29T17:27:08Z_

It's not really clear to me why we should have a full complement of _excess APIs in the first place. They originally existed to mirror jemalloc's experimental *allocm API, but those were removed in 4.0 several years ago in favor of not duplicating their entire API surface. It seems like we could follow their lead?

> Would it be possible to give alloc a default implementation in terms of alloc_excess later or is that a not possible or a breaking change?

We can add a default implementation of `alloc` in terms of `alloc_excess`, but `alloc_excess` will need to have a default implementation in terms of `alloc`. Everything works fine if you implement one or both, but if you don't implement either, your code will compile but infinitely recurse. This has come up before (maybe for `Rand`?), and we could have some way of saying that you need to implement at least one of those functions but we don't care which.

> It worries me a lot that I could not find any project that uses it as a collection type parameter (maybe I was just unlucky and there are some?).

I don't know of anyone that is doing that.


---

### Comment 208 by joshlf
_2017-11-29T22:04:32Z_

> The only projects that I've found that are using it are going to stay using nightly anyways (servo, redox)

One big thing preventing this from moving forward is that stdlib collections don't support parametric allocators yet. That pretty much precludes most other crates as well, since most external collections use internal ones under the hood (`Box`, `Vec`, etc).

---

### Comment 209 by remexre
_2017-11-30T03:46:48Z_

>> The only projects that I've found that are using it are going to stay using nightly anyways (servo, redox)
>
> One big thing preventing this from moving forward is that stdlib collections don't support parametric allocators yet. That pretty much precludes most other crates as well, since most external collections use internal ones under the hood (Box, Vec, etc).

This applies to me -- I've got a toy kernel, and if I could I'd be using `Vec<T, A>`, but instead I have to have an interior-mutable global allocator facade, which is gross.

---

### Comment 210 by sfackler
_2017-11-30T03:50:23Z_

@remexre how is parameterizing your data structures going to avoid global state with interior mutability?

---

### Comment 211 by remexre
_2017-11-30T03:55:14Z_

There will still be interior-mutable global state I suppose, but it feels a lot safer to have a setup where the global allocator is unusable until memory is fully mapped than to have a global `set_allocator` function.

---

**EDIT**: Just realized I didn't answer the question. Right now, I've got something like:

```
struct BumpAllocator{ ... }
struct RealAllocator{ ... }
struct LinkedAllocator<A: 'static + AreaAllocator> {
    head: Mutex<Option<Cons<A>>>,
}
#[global_allocator]
static KERNEL_ALLOCATOR: LinkedAllocator<&'static mut (AreaAllocator + Send + Sync)> =
    LinkedAllocator::new();
```

where `AreaAllocator` is a trait that lets me (at runtime) verify that the allocators aren't accidentally "overlapping" (in terms of the address ranges they allocate into). `BumpAllocator` is only used very early on, for scratch space when mapping the rest of memory to create the `RealAllocator`s.

Ideally, I'd like to have a `Mutex<Option<RealAllocator>>` (or a wrapper that makes it "insert-only") be the only allocator, and have everything allocated early on be parameterized by the early-boot `BumpAllocator`. This'd also let me ensure that the BumpAllocator doesn't get used after early-boot, since the stuff I allocate couldn't outlive it.

---

### Comment 212 by gnzlbg
_2017-11-30T08:40:34Z_

@sfackler 

> It's not really clear to me why we should have a full complement of _excess APIs in the first place. They originally existed to mirror jemalloc's experimental *allocm API, but those were removed in 4.0 several years ago in favor of not duplicating their entire API surface. It seems like we could follow their lead?

Currently `shrink_in_place` calls `xallocx` which returns the real allocation size. Because `shrink_in_place_excess` does not exist, it throws this size away, and users must call `nallocx` to recompute it, whose cost really depends on how big the allocation is.

So at least some jemalloc allocation functions that we are already using are returning us the usable size, but the current API does not allow us to use it. 

---

### Comment 213 by Ericson2314
_2017-11-30T16:14:45Z_

@remexre 

When I was working on my toy kernel, avoiding the global allocator to ensure no allocation happened until an allocator was set up was a goal of mine too. Glad to hear I'm not the one one!

---

### Comment 214 by jethrogb
_2017-11-30T18:56:47Z_

I do not like the word `Heap` for the default global allocator. Why not `Default`?

Another point of clarification: [RFC 1974](https://github.com/rust-lang/rfcs/blob/master/text/1974-global-allocators.md) puts all this stuff in `std::alloc` but it's currently in `std::heap`. Which location is being proposed for stabilization?

---

### Comment 215 by sfackler
_2017-11-30T19:20:33Z_

@jethrogb "Heap" is a pretty canonical term for "that thing malloc gives you pointers to" - what are your concerns with the term?

---

### Comment 216 by jethrogb
_2017-11-30T19:58:09Z_

@sfackler

> "that thing malloc gives you pointers to"

Except in my mind that is what `System` is.

---

### Comment 217 by sfackler
_2017-11-30T20:02:56Z_

Ah sure. `Global` is another name then maybe? Since you use `#[global_allocator]` to select it.

---

### Comment 218 by SimonSapin
_2017-11-30T22:31:18Z_

There can be multiple heap allocators (e.g. libc and prefixed jemalloc). How about renaming `std::heap::Heap` to `std::heap::Default` and `#[global_allocator]` to `#[default_allocator]`?

The fact that it’s what you get if you don’t specify otherwise (presumably when for example `Vec` gains an extra type parameter / field for the allocator) is more important than the fact that it doesn’t have "per-instances" state (or instances really).

---

### Comment 219 by rfcbot
_2017-12-01T22:58:28Z_

The final comment period is now complete.

---

### Comment 220 by SimonSapin
_2017-12-01T23:52:35Z_

Regarding FCP, I think that the API subset that was proposed for stabilization is of very limited use. For example, it does not support the `jemallocator` crate.

---

### Comment 221 by sfackler
_2017-12-02T00:30:10Z_

In what way? jemallocator may have to flag off some of the impls of unstable methods behind a feature flag but that's it.

---

### Comment 222 by SimonSapin
_2017-12-02T18:17:25Z_

If `jemallocator` on stable Rust cannot implement for example `Alloc::realloc` by calling `je_rallocx` but needs to rely on the default alloc + copy + dealloc impl, then it’s not an acceptable replacement for the standard library’s `alloc_jemalloc` crate IMO.

---

### Comment 223 by SimonSapin
_2017-12-02T18:18:04Z_

Sure, you could get *something* to compile, but it’s not a particularly useful thing.

---

### Comment 224 by sfackler
_2017-12-02T23:48:58Z_

Why? C++ doesn't have any concept of realloc at all in its allocator API and that doesn't appear to have crippled the language. It's obviously not ideal, but I don't understand why it would be unacceptable.

---

### Comment 225 by SimonSapin
_2017-12-03T08:39:42Z_

C++ collections generally don’t use realloc because C++ move constructors can run arbitrary code, not becuse realloc is not useful.

And the comparison is not with C++, it’s with the current Rust standard library with built-in jemalloc support. Switching to and out-of-std allocator with only this subset of `Alloc` API would be a regression.

And `realloc` is an example. jemallocator currently also implements `alloc_zeroed`, `alloc_excess`, `usable_size`, `grow_in_place`, etc.

---

### Comment 226 by sfackler
_2017-12-03T16:48:49Z_

alloc_zeroed is proposed to be stabilized. As far as I can tell (look upthread), there are literally zero uses of `alloc_excess` in existence. Could you show some code that will regress if that falls back to a default implementation.

More generally, though, I don't see why this is an argument against stabilizing a portion of these APIs. If you don't want to use jemallocator, you can continue not using it.

---

### Comment 227 by mzabaluev
_2017-12-04T12:29:18Z_

Could `Layout::array<T>()` be made a const fn?

---

### Comment 228 by sfackler
_2017-12-04T16:57:14Z_

It can panic, so not at the moment.

---

### Comment 229 by mzabaluev
_2017-12-04T17:57:45Z_

> It can panic, so not at the moment.

I see... I'd settle for `const fn Layout::array_elem<T>()` that would be a non-panicking equivalent of `Layout::<T>::repeat(1).0`.

---

### Comment 230 by joshlf
_2017-12-04T18:48:44Z_

@mzabaluev I think what you're describing is equivalent to [`Layout::new<T>()`](https://doc.rust-lang.org/nightly/alloc/allocator/struct.Layout.html#method.new). It can currently panic, but that's just because it's implemented using `Layout::from_size_align` and then `.unwrap()`, and I expect it could be done differently.

---

### Comment 231 by mzabaluev
_2017-12-04T19:21:34Z_

@joshlf I think this struct has the size of 5, while as elements of an array these are placed at every 8 bytes due to the alignment:
```rust
struct Foo {
    bar: u32,
    baz: u8
}
```
I'm not sure that an array of `Foo` would include the padding of the last element for its size calculation, but that's my strong expectation.

---

### Comment 232 by joshlf
_2017-12-04T19:29:29Z_

In Rust, the size of an object is always a multiple of its alignment so that the address of the `n`th element of an array is always `array_base_pointer + n * size_of<T>()`. So the size of an object in an array is always the same as the size of that object on its own. See the [Rustonomicon page on repr(Rust)](https://doc.rust-lang.org/nomicon/repr-rust.html) for more details.

---

### Comment 233 by mzabaluev
_2017-12-04T19:29:35Z_

OK, it turns out that a struct is padded to its alignment, but AFAIK this is not a stable guarantee except in `#[repr(C)]`.
Anyway, making `Layout::new` a const fn would be welcome as well.

---

### Comment 234 by SimonSapin
_2017-12-04T22:04:05Z_

This is the documented (and so guaranteed) behavior of a stable function:

https://doc.rust-lang.org/std/mem/fn.size_of.html
> Returns the size of a type in bytes.
>
> More specifically, this is the offset in bytes between successive elements in an array with that item type including alignment padding. Thus, for any type `T` and length `n`, `[T; n]` has a size of `n * size_of::<T>()`.

---

### Comment 235 by mzabaluev
_2017-12-05T05:13:14Z_

Thanks. I just realized that any const fn that multiplies the result of `Layout::new` would be inherently panicky in turn (unless it's done with `saturating_mul` or some such), so I'm back to square one. Continuing with a [question](/rust-lang/rust/issues/24111#issuecomment-349197148) about panics in the const fn tracking issue.

---

### Comment 236 by SimonSapin
_2017-12-05T09:06:30Z_

The `panic!()` macro is currently not supported in constant expressions, but panics from checked arithmetic are generated by the compiler and not affected by that limitation:

```rust
error[E0080]: constant evaluation error
 --> a.rs:1:16
  |
1 | const A: i32 = i32::max_value() * 2;
  |                ^^^^^^^^^^^^^^^^^^^^ attempt to multiply with overflow

error: aborting due to previous error
```

---

### Comment 237 by gnzlbg
_2017-12-08T16:25:15Z_

This is related to `Alloc::realloc` but not to stabilization of the minimal interface (`realloc` is not part of it):

Currently, because `Vec::reserve/double` call `RawVec::reserve/double` which call `Alloc::realloc`, the default impl of `Alloc::realloc` copies dead vector elements (in the `[len(), capacity())` range). In the absurd case of a huge empty vector that wants to insert `capacity() + 1` elements and thus reallocates, the cost of touching all that memory is not insignificant.

In theory, if the default `Alloc::realloc` implementation would also take a "bytes_used" range, it could just copy the relevant part on reallocation. In practice, at least jemalloc overrides `Alloc::realloc` default impl with a call to `rallocx`. Whether doing an `alloc`/`dealloc` dance copying only the relevant memory is faster or slower than a `rallocx` call will probably depend on many things (does `rallocx` manage to expand the block in place? how much unnecessary memory will `rallocx` copy? etc.). 

---

### Comment 238 by Ericson2314
_2017-12-28T03:44:25Z_

https://github.com/QuiltOS/rust/tree/allocator-error I've started to demonstrate how I think associated error type solves our collections and error-handling problems by doing the generalization itself. In particular, note how in the modules I change that I
 - Always reuse the `Result<T, A::Err>` implementation for the `T` implemetation
 - Never `unwrap` or anything else partial
 - No `oom(e)` outside of `AbortAdapter`.

This means that the changes I'm making are quite safe, and quite mindless too! Working with both the error-returning and error-aborting should not require extra effort to maintain mental invariants---the type checker does all the work.

I recall---I think in @Gankro's RFC? or the pre-rfc thread---Gecko/Servo people saying it was nice to not have the fallibility of collections be part of their type. Well, I can add a `#[repr(transparent)]` to `AbortAdapter` so that collections can safely be transmuted between `Foo<T, A>` and `Foo<T, AbortAdapter<A>>` (within safe wrappers), allowing one to freely switch back and forth without duplicating every method. [For back-compat, the standard library collections will need to be duplicated in any event, but user methods need not be as `Result<T, !>` is these days quite easy to work with.] 

~~Unfortunately the code won't fully type check because changing the type params of a lang item (box) confuses the compiler (surprise!). But hopefully what I have so far gives a flavor of what I am doing. The ICE-causing box commit is the last one---everything before it is good.~~ @eddyb fixed rustc in #47043!

*edit* @joshlf I was informed of your https://github.com/rust-lang/rust/pull/45272, and incorporated that in here. Thanks!

---

### Comment 239 by raphaelcohn
_2018-01-04T18:36:09Z_

Persistent Memory (eg <http://pmem.io>) is the next big thing, and Rust needs to be positioned to work well with it.

I've been working recently on a Rust wrapper for a persistent memory allocator (specifically, libpmemcto). Whatever decisions are made regarding the stabilisation of this API, it needs to:-

* Be able to support a performant wrapper around a persistent memory allocator like libpmemcto;
* Be able to specify (parameterize) collection types by allocator (at the moment, one needs to duplicate Box, Rc, Arc, etc)
* Be able to clone data across allocators
* Be able to support having persistent memory stored structs with fields that are re-initialized on instantiation of a persistent memory pool, ie, some persistent memory structs needs to have fields that are only stored temporarily on the heap. My current use cases are a reference to the persistent memory pool used for allocation and transient data used for locks.

As an aside, the pmem.io development (Intel's PMDK) makes heavy use of a modified jemalloc allocator under the covers - so it seems prudent that using jemalloc as an example API consumer would be prudent.

---

### Comment 240 by gnzlbg
_2018-01-16T12:10:01Z_

Would it be possible to reduce the scope of this to cover only `GlobalAllocator`s first until we gain more experience with using `Alloc`ators in collections? 

IIUC this already would serve `servo`'s needs and would allow us to experiment with parametrizing containers in parallel. In the future we can either move collections to use `GlobalAllocator` instead or just add a blanket impl of `Alloc` for `GlobalAllocator` so that these can be used for all collections.

Thoughts?

---

### Comment 241 by SimonSapin
_2018-01-16T12:41:03Z_

@gnzlbg For the `#[global_allocator]` attribute to be useful (beyond selecting `heap::System`) the `Alloc` trait needs to be stable too, so that it can be implemented by crates like https://crates.io/crates/jemallocator. There is no type or trait named `GlobalAllocator` at the moment, are you proposing some new API?

---

### Comment 242 by gnzlbg
_2018-01-16T13:01:47Z_

> here is no type or trait named GlobalAllocator at the moment, are you proposing some new API?

What I suggested is renaming the "minimal" API that @alexcrichton suggested to stabilize [here](https://github.com/rust-lang/rust/issues/32838#issuecomment-336957415) from `Alloc` to `GlobalAllocator` to represent only global allocators, and leaving the door open for collections to be parametrized by a different allocator trait in the future (which does not mean that we can't parametrize them by the `GlobalAllocator` trait).

IIUC `servo` currently only needs to be able to switch the global allocator (as opposed to also being able to parametrize some collections by an allocator). So maybe instead of trying to stabilize a solution that should be future proofed for both use-cases, we can address only the global allocator issue now, and figure out how to parametrize collections by allocators later. 

Don't know if that makes sense.

---

### Comment 243 by SimonSapin
_2018-01-16T13:26:42Z_

> IIUC servo currently only needs to be able to switch the global allocator (as opposed to also being able to parametrize some collections by an allocator).

That is correct, but:

* If a trait and its method are stable so that it can be implemented, then it can also be called directly without going through `std::heap::Heap`. So it’s not only a trait global allocator, it’s a trait for allocators (even if we end up making a different one for collections generic over allocators) and `GlobalAllocator` is not a particularly good name.
* The jemallocator crate currently implements `alloc_excess`, `realloc`, `realloc_excess`, `usable_size`, `grow_in_place`, and `shrink_in_place` which are not part the proposed minimal API. These can be more efficient than the default impl, so removing them would be a performance regression.

---

### Comment 244 by gnzlbg
_2018-01-16T13:39:15Z_

Both points make sense. I just thought that the only way to significantly accelerate the stabilization of this feature was to cut out a dependency on it also being a good trait for parametrizing collections over it. 

---

### Comment 245 by Ericson2314
_2018-01-16T18:23:07Z_

[It would be nice if Servo could be like (stable | official mozilla crate), and cargo could enforce this, to remove a little pressure here.]

---

### Comment 246 by sfackler
_2018-01-16T18:32:29Z_

@Ericson2314 servo is not the only project that wants to use these APIs.

---

### Comment 247 by SimonSapin
_2018-01-17T07:24:04Z_

@Ericson2314 I don’t understand what this means, could you rephrase?

---

### Comment 248 by SimonSapin
_2018-01-17T08:09:24Z_

For context: Servo currently uses a number unstable features (including `#[global_allocator]`), but we’re trying to slowly move away from that (either by updating to a compiler that has stabilized some features, or by finding stable alternatives.) This is tracked at https://github.com/servo/servo/issues/5286. So stabilizing `#[global_allocator]` would be nice, but it’s not blocking any Servo work.

Firefox relies on the fact that Rust std defaults to the system allocator when compiling a `cdylib`, and that mozjemalloc which ends up being linked into the same binary defines symbols like `malloc` and `free` that "shadow" (I don’t know the proper linker terminology) those from libc. So allocations from Rust code in Firefox ends up using mozjemalloc. (This is on Unix, I don’t know how it works on Windows.) This works out, but it feels fragile to me. Firefox uses stable Rust, and I’d like it to use `#[global_allocator]` to explicitly select mozjemalloc to make the whole setup is more robust.

---

### Comment 249 by gnzlbg
_2018-01-17T10:47:45Z_

@SimonSapin the more that I play with allocators and collections, the more I tend to think that we don't want to parametrize the collections by `Alloc` yet, because depending on the allocator, a collection might want to offer a different API, the complexity of some operations change, some collection details actually depend on the allocator, etc. 

So I would like to suggest a way in which we can make progress here. 

## Step 1: Heap allocator

We could restrict ourselves at first to try to let users select the allocator for the heap (or the system/platform/global/free-store allocator, or however you prefer to name it) in stable Rust. 

The only thing that we initially parametrize by it is `Box`, which only needs to allocate (`new`) and deallocate (`drop`) memory. 

This allocator trait could initially have the API that @alexcrichton proposed (or somewhat extended), and this allocator trait could, on nightly, still have a slightly extended API to support the `std::` collections.

Once we are there, users that want to migrate to stable will be able to do so, but might get a performance hit, because of the unstable API. 

## Step 2: Heap allocator without performance hit

At that point, we can re-evaluate the users that can't move to stable because of a performance hit, and decide how to extend this API and stabilize that. 

## Steps 3 to N: supporting custom allocators in `std` collections.

First, this is hard, so it might never happen, and I think it never happening isn't a bad thing. 

When I want to parametrize a collection with a custom allocator I either have a performance problem or an usability problem. 

If I have an usability problem I typically want a different collection API that exploits features of my custom allocator, like for example my `SliceDeque` crate does. Parametrizing a collection by a custom allocator won't help me here.

If I have a performance problem, it would still be very hard for a custom allocator to help me. I am going to consider `Vec` in the next sections only, because it is the collection I've reimplemented most often. 

### Reduce the number of system allocator calls (Small Vector Optimization)

If I want to allocate some elements inside the `Vec` object to reduce the number of calls to the system allocator, today I just use `SmallVec<[T; M]>`. However, a `SmallVec` is not a `Vec`:

*  moving a `Vec` is O(1) in the number of elements, but moving a `SmallVec<[T; M]>` is O(N) for N < M and O(1) afterwards,

* pointers to the `Vec` elements are invalidated on move if `len() <= M` but not otherwise, that is, if `len() <= M` operations like `into_iter` need to move the elements into the iterator object itself, instead of just taking pointers. 

Could we make `Vec` generic over an allocator to support this? Everything is possible, but I think that the most important costs are:

* doing so makes the implementation of `Vec` more complex, which might impact users not using this feature
* the documentation of `Vec` would become more complex, because the behavior of some operations would depend on the allocator. 

I think these costs are non-negligible. 

### Make use of allocation patterns

The growth-factor of a `Vec` is tailored to a particular allocator. In `std` we can tailor it to the common ones `jemalloc`/`malloc`/... but if you are using a custom allocator, chances are that the growth-factor we choose by default won't be the best for your use case. Should every allocator be able to specify a growth-factor for vec-like allocation patterns? I don't know but my gut feeling tells me: probably not.

### Exploit extra features of your system allocator

For example, an over-committing allocator is available in most of the Tier 1 and Tier 2 targets. In Linux-like and Macos systems the heap allocator overcommits by default, while the Windows API exposes `VirtualAlloc` which can be used to reserve memory (e.g. on `Vec::reserve/with_capacity`) and commit memory on `push`. 

Currently the `Alloc` trait doesn't expose a way to implement such an allocator on Windows, because it doesn't separate the concepts of commiting and reserving memory (on Linux a non-over-commiting allocator can be hacked in by just touching each page once). It also doesn't expose a way for an allocator to state whether it over-commits or not by default on `alloc`. 

That is, we would need to extend the `Alloc` API to support this for `Vec`, and that would be IMO for little win. Because when you have such an allocator, `Vec` semantics change again:

* `Vec` doesn't need to grow ever again, so operations like `reserve` make little sense
* `push` is `O(1)` instead of amortized `O(1)`.
* iterators to live objects are never invalidates as long as the object is alive, which allows some optimizations 

### Exploit more extra features of your system allocator

Some system alloctors like `cudaMalloc`/`cudaMemcpy`/... differentiate between pinned and non-pinned memory, allow you to allocate memory on disjoint address spaces (so we would need an associated Pointer type in the Alloc trait), ...

But using these on collections like Vec does again change the semantics of some operations in subtle ways, like whether indexing a vector suddenly invokes undefined behavior or not, depending on whether you do so from a GPU kernel or from the host. 

## Wrapping up

I think that trying to come up with an `Alloc` API that can be used to parametrize all collections (or only even `Vec`) is hard, probably too hard. 

Maybe after we get global/system/platform/heap/free-store allocators right, and `Box`, we can rethink the collections. Maybe we can reuse `Alloc`, maybe we need a `VecAlloc,` VecDequeAlloc`, `HashMapAlloc`, ... or maybe we just say, "you know what, if you really need this, just copy-paste the standard collection into a crate, and mold it to your allocator". Maybe the best solution is to just make this easier, by having std collections in its own crate (or crates) in the nursery and using only stable features, maybe implemented as a set of building blocks. 

Anyways, I think trying to tackle all these problems here at once and trying to come up with an `Alloc` trait that is good for everything is too hard. We are at step 0. I think that the best way to get to step 1 and step 2 quick is to leave collections out of the picture until we are there. 

---

### Comment 250 by SimonSapin
_2018-01-17T14:12:40Z_

> Once we are there, users that want to migrate to stable will be able to do so, but might get a performance hit, because of the unstable API.

Picking a custom allocator is usually about improving performance, so I don’t know who this initial stabilization would serve.

---

### Comment 251 by gnzlbg
_2018-01-17T14:18:10Z_

> Picking a custom allocator is usually about improving performance, so I don’t know who this initial stabilization would serve.

Everybody? At least right now. ~~Most~~ Some of the methods you complain are missing in the initial stabilization proposal (`alloc_excess`, for example), are AFAIK not used by anything in the standard library yet. Or did this change recently?

---

### Comment 252 by SimonSapin
_2018-01-17T14:24:50Z_

`Vec` (and other users of `RawVec`) use `realloc` in `push`

---

### Comment 253 by gnzlbg
_2018-01-17T14:25:45Z_

@SimonSapin  

> The jemallocator crate currently implements alloc_excess, realloc, realloc_excess, usable_size, grow_in_place, and shrink_in_place 

From these methods, AFAIK `realloc`, `grow_in_place`, and `shrink_in_place` are used but `grow_in_place` is only a naive wrapper over `shrink_in_place` for jemalloc at least so if we implemented the default unstable impl of `grow_in_place` in terms of `shrink_in_place` in the `Alloc` trait, that cuts it down to two methods: `realloc` and `shrink_in_place`.

> Picking a custom allocator is usually about improving performance,

While this is true, you might get more performance from using a more suited allocator without these methods, than a bad allocator that has them. 

IIUC the main use case for servo was to use Firefox jemalloc instead of having a second jemalloc around, was that right?

---

### Comment 254 by gnzlbg
_2018-01-17T14:34:20Z_

Even if we add `realloc` and `shrink_in_place` to the `Alloc` trait in an initial stabilization, that would only delay the performance complaints. 

For example, the moment we add any unstable API to the `Alloc` trait that ends up being using by the `std` collections,  you wouldn't be able to get the same performance on stable than you would be able to get on nightly. That is, if we add `realloc_excess` and `shrink_in_place_excess` to the alloc trait and make `Vec`/`String`/... use them, that we stabilized `realloc` and `shrink_in_place` wouldn't have helped you a single bit.

---

### Comment 255 by SimonSapin
_2018-01-17T14:40:39Z_

> IIUC the main use case for servo was to use Firefox jemalloc instead of having a second jemalloc around, was that right?

Although they share some code, Firefox and Servo are two separate projects/applications.

Firefox use mozjemalloc, which is a fork of an old version of jemalloc with a bunch of features added. I *think* that some `unsafe` FFI code relies for correctness and soundness on mozjemalloc being used by Rust std.

Servo uses jemalloc which happens to be Rust’s default for executables at the moment, but there are plans to change that default to the system’s allocator. Servo also has some `unsafe` memory usage reporting code that relies for soundness on jemalloc being used indeed. (Passing `Vec::as_ptr()` to `je_malloc_usable_size`.)

---

### Comment 256 by gnzlbg
_2018-01-17T14:50:26Z_

> Servo uses jemalloc which happens to be Rust’s default for executables at the moment, but there are plans to change that default to the system’s allocator.

It would be good to know if the system allocators in the systems that servo targets provide optimized `realloc` and `shrink_to_fit` APIs like jemalloc does? `realloc` (and `calloc`) are very common, but `shrink_to_fit` (`xallocx`) is AFAIK specific to `jemalloc`. Maybe the best solution would be to stabilize `realloc` and `alloc_zeroed` (`calloc`) in the initial implementation, and leave `shrink_to_fit` for later. That should allow servo to work with system allocators in most platforms without performance issues. 

> Servo also has some unsafe memory usage reporting code that relies for soundness on jemalloc being used indeed. (Passing Vec::as_ptr() to je_malloc_usable_size.)

As you know, the `jemallocator` crate has APIs for this. I expect that crates similar to the `jemallocator` crate will pop up for other allocators offering similar APIs as the global allocator story begins getting stabilized. I haven't thinked about whether these APIs belong in the `Alloc` trait at all.



---

### Comment 257 by SimonSapin
_2018-01-17T14:55:49Z_

I don’t think `malloc_usable_size` needs to be in the `Alloc` trait. Using `#[global_allocator]` to be confident what allocator is used by `Vec<T>` and separately using a function from the `jemallocator` crate is fine.

---

### Comment 258 by gnzlbg
_2018-01-17T15:07:34Z_

@SimonSapin once the `Alloc` trait becomes stable, we'll probably have a crate like `jemallocator` for Linux malloc and Windows. These crates could have an extra feature to implement the parts that they can of the unstable `Alloc` API (like, e.g., `usable_size` on top of `malloc_usable_size`) and some other things that are not part of the `Alloc` API, like memory reporting on top of `mallinfo`. Once there are usable crates for the systems that servo targets it would be easier to know which parts of the `Alloc` trait to prioritize stabilizing, and we'll probably find out newer APIs that should be at least experimented with for some allocators.

---

### Comment 259 by Ericson2314
_2018-01-17T21:11:50Z_

@gnzlbg I'm a bit skeptical of the things in https://github.com/rust-lang/rust/issues/32838#issuecomment-358267292. Leaving out all those system-specific stuff, it's not hard to generalize collections for alloc--I've done it. Trying to incorporate that seems like a separate challenge.

@SimonSapin Does firefox have a no unstable Rust policy? I think I was getting confused: Firefox and Servo want this, but if so its Firefox's use-case that would be adding pressure to stabilize.

@sfackler See that ^. I was trying to make a distinction between projects that need vs want this stable, but Servo is on the other side of that divide.

---

### Comment 260 by sfackler
_2018-01-17T21:15:13Z_

I have a project that wants this and requires it to be stable. There is nothing particularly magical about either Servo or Firefox as consumers of Rust.

---

### Comment 261 by SimonSapin
_2018-01-17T21:26:05Z_

@Ericson2314 Correct, Firefox uses stable: https://wiki.mozilla.org/Rust_Update_Policy_for_Firefox. As I explained though there’s working solution today, so this is not a real blocker for anything. It’d be nicer / more robust to use `#[global_allocator]`, that’s all.

Servo does use some unstable features, but as mentioned we’re trying to change that.

---

### Comment 262 by joshlf
_2018-01-17T21:31:15Z_

FWIW, parametric allocators are very useful to implement allocators. A lot of the less performance-sensitive bookkeeping becomes much easier if you can use various data structures internally and parametrize them by some simpler allocator (like [bsalloc](https://crates.io/crates/bsalloc)). Currently, the only way to do that in a std environment is to have a two-phase compilation in which the first phase is used to set your simpler allocator as the global allocator and the second phase is used to compile the larger, more complicated allocator. In no-std, there's no way to do it at all.

---

### Comment 263 by gnzlbg
_2018-01-18T09:10:46Z_

@Ericson2314 

> Leaving out all those system-specific stuff, it's not hard to generalize collections for alloc--I've done it. Trying to incorporate that seems like a separate challenge.

Do you have an implementation of `ArrayVec` or `SmallVec` on top of `Vec` + custom allocators that I could look at? That was the first point I mentioned, and that's not system specific at all. Arguably that would be the simplest two allocators imaginable, one is just a raw array as storage, and the other one can be built on top of the first one by adding a fallback to the Heap once the array runs out of capacity. The main difference is that those allocators are not "global", but each of the `Vec`s has its own allocator independent of all others, and these allocators are stateful.

Also, I am not arguing to never do this. I am just stating that this is very hard: C++ has been trying for 30 years with only partial-success: GPU allocators and GC allocators work due to the generic pointer types, but implementing `ArrayVec` and `SmallVec` on top of `Vec` does not result in a zero-cost abstraction in C++ land ([P0843r1](https://github.com/gnzlbg/fixed_capacity_vector) discusses some of the issues for `ArrayVec` in detail). 

So I'd just prefer if we would pursue this after we stabilize the pieces that do deliver something useful as long as these don't make pursuing custom collection allocators in the future.

---

I talked a bit with @SimonSapin on IRC and if we were to extend the initial stabilization proposal with `realloc` and `alloc_zeroed`, then Rust in Firefox (which only uses stable Rust) would be able to use `mozjemalloc` as a global allocator in stable Rust without the need for any extra hacks. As mentioned by @SimonSapin, Firefox currently has a workable solution for this today, so while this would be nice, it doesn't seem to be very high priority.

Still, we could start there, and once we are there, move `servo` to stable `#[global_allocator]` without a performance loss.

---

@joshlf 

> FWIW, parametric allocators are very useful to implement allocators.

Could you elaborate a bit more on what you mean? Is there a reason why you can't parametrize your custom allocators with the `Alloc` trait? Or your own custom allocator trait and just implement the `Alloc` trait on the final allocators (these two traits do not necessarily need to be equal)?

---

### Comment 264 by hanna-kruppe
_2018-01-18T10:49:10Z_

I don't understand where the use case of "SmallVec = Vec + special allocator" comes from. It's not something I've seen mentioned a lot before (neither in Rust nor in other contexts), precisely because it has many serious problems. When I think of "improving performance with a specialized allocator", that's not at all what I think of.

---

### Comment 265 by rolandsteiner
_2018-01-18T11:09:54Z_

Looking over the `Layout` API, I was wondering about the differences in error handling between `from_size_align` and `align_to`, where the former returns `None` in case of an error, while the latter panics (!).

Wouldn't it be more helpful and consistent to add a suitably defined and informative `LayoutErr` enum and return a `Result<Layout, LayoutErr>` in both cases (and perhaps use it for the other functions that currently return an `Option` as well)?

---

### Comment 266 by gnzlbg
_2018-01-18T13:40:18Z_

@rkruppe 

> I don't understand where the use case of "SmallVec = Vec + special allocator" comes from. It's not something I've seen mentioned a lot before (neither in Rust nor in other contexts), precisely because it has many serious problems. When I think of "improving performance with a specialized allocator", that's not at all what I think of.

There are two independent ways of using allocators in Rust and C++: the system allocator, used by all allocations by default, and as a type argument for a collection parametrized by some allocator trait, as a way to create an object of that particular collection that uses a particular allocator (which can be the system's allocator or not).

What follows focus only on this second use case: using a collection and an allocator type to create an object of that collection that uses a particular allocator.

In my experience with C++, parametrizing a collection with an allocator serves two use cases: 

* improve performance of a collection object by making the collection use a custom allocator targeted at a specific allocation pattern, and/or  
* add a new feature to a collection allowing it to do something that it couldn't do before.

## Adding new features to collections

This is the use case of allocators that I see in C++ code-bases 99% of the time. The fact that adding a new feature to a collection improves performance is, in my opinion, coincidental. In particular, none of the following allocators improves performance by targeting an allocation pattern. They do so by adding features, that in some cases, as @Ericson2314 mentions, can be considered "system-specific".  These are some examples:

* stack allocators for doing small buffer optimizations (see Howard Hinnant's [stack_alloc paper](https://howardhinnant.github.io/stack_alloc.html)). They let you use `std::vector` or `flat_{map,set,multimap,...}` and by passing it a custom allocator you add in a small buffer optimization with (`SmallVec`) or without (`ArrayVec`) heap fall back. This allows, for example, putting a collection with its elements on the stack or static memory (where it would have otherwise used the heap). 

* segmented memory architectures (like 16-bit wide pointer x86 targets and GPGPUs). For example, C++17 Parallel STL was, during C++14, the Parallel Technical Specification. It's precursor library of the same author is NVIDIA's Thrust library, which includes allocators to allow container clases to use GPGPU memory (e.g. [thrust::device_malloc_allocator](https://github.com/thrust/thrust/blob/master/thrust/device_malloc_allocator.h)) or pinned memory (e.g. [thrust::pinned_allocator](https://github.com/thrust/thrust/blob/8e35dce3d2003f9e9f7eed8dac31be7f050c8b2d/thrust/system/cuda/experimental/pinned_allocator.h); pinned memory allows faster transfer between host-device in some cases).  

* allocators to solve parallelism-related issues, like false sharing (e.g. Intel Thread Building Blocks [`cache_aligned_allocator`](https://software.intel.com/en-us/node/506260)) or over-alignment requirements of SIMD types (e.g. Eigen3's [`aligned_allocator`](https://eigen.tuxfamily.org/dox/classEigen_1_1aligned__allocator.html)).  

* interprocess shared memory: [Boost.Interprocess](http://www.boost.org/doc/libs/1_51_0/doc/html/interprocess/allocators_containers.html#interprocess.allocators_containers.allocator_introduction.allocator_properties) has allocators that allocate the collection's memory using OS interprocess shared memory facilities (e.g. like System V shared memory). This allows to directly use a std container to manage memory used to communicate between different processes.

* garbage collection: Herb Sutter's [deferred memory allocation library](https://github.com/hsutter/gcpp) uses a user-defined pointer type to implement allocators that garbage collect memory. So that, for example, when a vector grows, the old chunk of memory is maintained alive till all pointers to that memory have been destroyed, avoiding iterator invalidation. 

* instrumented allocators: [Bloomberg's Software Library's blsma_testallocator](https://github.com/bloomberg/bde/blob/master/groups/bsl/bslma/bslma_testallocator.h) allows you to log memory alloctation/deallocation (and C++-specific object construction/destruction) patterns of the objects where you use it. You don't know if a `Vec` allocates after `reserve` ? Plug in such an allocator, and they will tell you if it happens. Some of these allocators let you name them, so that you can use it on multiple objects and get logs saying which object is doing what.

These are the types of allocators that I see most often in the wild in C++. As I mentioned before, the fact that they improve performance in some cases, is, in my opinion, coincidental. The important part is that none of them tries to target a particular allocation pattern.

##  Targeting allocation patterns to improve performance.

AFAIK there aren't any widely used C++ allocators that do this and I'll explain why I think this is in a second. The following libraries target this use case:

* C++ [Boost.Pool](http://www.boost.org/doc/libs/1_45_0/libs/pool/doc/interfaces/pool_alloc.html),
* C++ [foonathan/memory](https://github.com/foonathan/memory)
* D's std.experimental.allocator

However, these libraries do not really provide a single allocator for a particular use case. Instead, they provide allocator building blocks that you can use to build custom allocators targeted at the particular allocation pattern in a particular part of an application. 

The general advice that I recall from my C++ days was to just "don't use them" (they are the very last resort) because:

* matching the system allocator's performance is very hard, beating it is very very hard, 
* the chances of someone else's application memory allocation pattern matching yours is slim, so you really need to know your allocation pattern and know what allocator building blocks you need to match it
* they are not portable because different vendors have different C++ standard library implementations which do use different allocation patterns; vendors typically target their implementation at their system allocators. That is, a solution tailored to one vendor might perform horribly (worse than the system allocator) in another. 
* there are many alternatives one can exhaust before trying to use these: using a different collection, reserving memory, ... Most of the alternatives are lower effort and can deliver larger wins.

This does not mean that libraries for this use case aren't useful. They are, which is why libraries like [foonathan/memory](https://github.com/foonathan/memory) are blooming. But at least in my experience they are way less used in the wild than allocators that "add extra features" because to deliver a win you must beat the system allocator, which is requires more time than most users are willing to invest (Stackoverflow is full of questions of the type "I used Boost.Pool and my performance got worse, what can I do? Not use Boost.Pool."). 

## Wrapping up

IMO I think it is great that the C++ allocator model, though far from perfect, supports both use cases, and I think that if Rust's std collections are to be parametrized by allocators, they should support both use cases as well, because at least in C++ allocators for both cases have turned out to be useful.

I just think this problem is slightly orthogonal to being able to customize the global/system/platform/default/heap/free-store allocator of a particular application, and that trying to solve both problems at the same time might delay the solution for one of them unnecessarily. 

What some users want to do with a collection parametrized by an allocator might be way different from what some other users want to do. If @rkruppe starts from "matching allocation patterns" and I start from "preventing false sharing" or "using a small buffer optimization with heap fallback"  it's just going to be hard to first, understand the needs of each other, and second, arrive at a solution that works for both. 

---

### Comment 267 by hanna-kruppe
_2018-01-18T14:06:51Z_

@gnzlbg Thanks for the comprehsive write-up. Most of it doesn't address my original question and I disagree with some of it, but it's good to have it spelled out so we don't talk past each other.

My question was specifically about this application:

> stack allocators for doing small buffer optimizations (see Howard Hinnant's stack_alloc paper). They let you use std::vector or flat_{map,set,multimap,...} and by passing it a custom allocator you add in a small buffer optimization with (SmallVec) or without (ArrayVec) heap fall back. This allows, for example, putting a collection with its elements on the stack or static memory (where it would have otherwise used the heap).

Reading about stack_alloc, I realize now how that can work. It's not what people usually mean by SmallVec (where the buffer is stored inline in the collection), which is why I missed that option, but it side steps the problem of having to update pointers when the collection moves (and also makes those moves cheaper). Also note that short_alloc allows multiple collections to share one `arena`, which makes it even more unlike typical SmallVec types. It's more like a linear/bump-pointer allocator with graceful fallback to heap allocation when running of out alotted space.

I disagree that this sort of allocator and `cache_aligned_allocator` are fundamentally adding new features. They are *used* differently, and depending on your definition of "allocation pattern" they may not optimize for a specific allocation pattern. However, they certainly optimize for specific use cases and they don't have any significant behavioral differences from general-purpose heap allocators.

I do however agree that use cases like Sutter's deferred memory allocation, which substantially change what a "pointer" even means, are a separate application that may need a separate design if we want to support it at all.

---

### Comment 268 by gnzlbg
_2018-01-18T15:21:14Z_

> Reading about stack_alloc, I realize now how that can work. It's not what people usually mean by SmallVec (where the buffer is stored inline in the collection), which is why I missed that option, but it side steps the problem of having to update pointers when the collection moves (and also makes those moves cheaper).

I mentioned `stack_alloc` because it is the only such allocator with "a paper", but it was released in 2009 and precedes C++11 (C++03 did not support stateful allocators in collections). 

The way this work in C++11 (which supports stateful allocators), in a nutshell, is:  

* std::vector stores an `Allocator` object [inside it](https://github.com/llvm-mirror/libcxx/blob/master/include/vector#L334) just like Rust `RawVec` [does](https://github.com/rust-lang/rust/blob/master/src/liballoc/raw_vec.rs#L50).
* the [Allocator interface](http://en.cppreference.com/w/cpp/memory/allocator_traits) has an obscure property called [Allocator::propagate_on_container_move_assignment](http://en.cppreference.com/w/cpp/memory/allocator_traits) (POCMA from now on) that user-defined allocators can customize; this property is `true` by default. If this property is `false`, on move assignment, the allocator cannot be propagated, so a collection is required by the standard to move each of its elements to the new storage manually.

So when a vector with the system allocator is moved, first the storage for the new vector on the stack is allocated, then the allocator is moved (which is zero-sized), and then the 3 pointers are moved, which are still valid. Such moves are `O(1)`. 

OTOHO, when a vector with a `POCMA == true` allocator is moved, first the storage for the new vector on the stack is allocated and initialized with an empty vector, then the old collection is `drain`ed into the new one, so that the old one is empty, and the new one full. This moves each element of the collection individually, using their move assignment operators. This step is `O(N)` and fixes internal pointers of the elements. Finally, the original now empty collection is dropped. Note that this looks like a clone, but isn't because the elements themselves aren't cloned, but moved in C++. 

Does that make sense?

The main problem with this approach in C++ are that:

* the vector growth-policy is implementation defined
* the allocator API does not have `_excess` methods
* the combination of the two issues above means that if you know your vector can at most hold 9 elements, you can't have a stack allocator that can hold 9 elements, because your vector might try to grow when it has 8 with a growth factor of 1.5 so you need to pessimize and allocate space for 18 elements. 
* the complexity of the vector operation changes depending on allocator properties (POCMA is just one of many properties that the C++ Allocator API has; writing C++ allocators is non-trivial). This makes specifying the API of vector a pain because sometimes copying or moving elements between different allocators of the same type has extra costs, which change the complexity of the operations. It also makes reading the spec a huge pain. Many online sources of documentation like cppreference put the general case upfront, and the obscure details of what changes if one allocator property is true or false in tiny small letters to avoid bothering 99% of the users with them.

There are many people working on improving C++'s allocator API to fix these issues, for example, by adding `_excess` methods and guaranteeing that standard conforming collections use them.

> I disagree that this sort of allocator and cache_aligned_allocator are fundamentally adding new features.

Maybe what I meant is that they allow you to use std collections in situations or for types for which you couldn't use them before. For example, in C++ you can't put the elements of a vector in the static memory segment of your binary without something like a stack allocator (yet you can write your own collection that does it). OTOH, the C++ standard does not support over-aligned types like SIMD types, and if you try to heap allocate one with `new` you will invoke undefined behavior (you need to use `posix_memalign` or similar). Using the object typically manifests the undefined behavior via a segfault (*). Things like `aligned_allocator` allow you to heap allocate these types, and even put them in std collections, without invoking undefined behavior, by using a different allocator. Sure the new allocator will have different allocation patterns (these allocators basically overalign all memory btw...), but what people use them for is to be able to do something that they couldn't do before. 

Obviously, Rust is not C++. And C++ has problems that Rust doesn't have (and vice-versa). An allocator that adds a new feature in C++ might be unnecessary in Rust, which, for example, doesn't have any problems with SIMD types. 

(*) Users of Eigen3 suffer this deeply, because to avoid undefined behavior when using C++ and STL containers, you need to guard the containers against SIMD types, or types containing SIMD types ([Eigen3 docs](http://eigen.tuxfamily.org/dox/group__TopicStlContainers.html)) and also you need to guard your self from ever using `new` on your types by overloading operator `new` for them ([more Eigen3 docs](https://eigen.tuxfamily.org/dox/group__TopicStructHavingEigenMembers.html)). 

---

### Comment 269 by Ericson2314
_2018-01-18T16:59:19Z_

@gnzlbg thanks, I was also confused by the smallvec exmaple. That would require non-moveable types and some sort of alloca in Rust---two RFCs in review and then more follow-up work---so I have no qualms punting on that for now. The existing smallvec strategy of always using all the stack space you'll need seems fine for now.

I also agree with @rkruppe that in your revised list, the new capabilities of the allocator need *not* be known to the collection using the allocator. Sometimes the full `Collection<Allocator>` has new properties (existing entirely in pinned memory, lets say) but that's just a natural consequence of using the allocator.

The one exception here I see is allocators that only allocate a single size/type (the NVidia ones do this, as do slab allocators). We could have a separate `ObjAlloc<T>` trait that's blanket implemented for normal allocators: `impl<A: Alloc, T> ObjAlloc<T> for A`. Then, collections would use ObjAlloc bounds if they just needed to allocate a few items. But, I feel somewhat silly even bringing this up as it should be doable backwards compatibly later.

---

### Comment 270 by hanna-kruppe
_2018-01-18T17:08:21Z_

> Does that make sense?

Sure but it's not really relevant to Rust since we have no move constructors. So a (movable) allocator that directly contains the memory it hands out pointers to just isn't possible, period.

> For example, in C++ you can't put the elements of a vector in the static memory segment of your binary without something like a stack allocator (yet you can write your own collection that does it).

This is not a behavioral change. There are many valid reasons to control where collections get their memory, but all of them related to "externalities" like performance, linker scripts, control over the whole program's memory layout, etc.

> Things like aligned_allocator allow you to heap allocate these types, and even put them in std collections, without invoking undefined behavior, by using a different allocator.

This is why I specifically mentioned TBB's cache_aligned_allocator and not Eigen's aligned_allocator. cache_aligned_allocator does not seem to guarantee any specific alignment in its documentation (it just says that it's "typically" 128 byte), and even if it did it usually wouldn't be used for this purpose (because its alignment is probably too large for common SIMD types and too small for things like page-aligned DMA). Its purpose, as you state, is to avoid false sharing.

---

### Comment 271 by joshlf
_2018-01-18T17:16:54Z_

@gnzlbg 

> > FWIW, parametric allocators are very useful to implement allocators.
> 
> Could you elaborate a bit more on what you mean? Is there a reason why you can't parametrize your custom allocators with the Alloc trait? Or your own custom allocator trait and just implement the Alloc trait on the final allocators (these two traits do not necessarily need to be equal)?

I think I wasn't clear; let me try to explain better. Let's say I'm implementing an allocator that I expect to use either:
- As the global allocator
- In a no-std environment

And let's say that I'd like to use `Vec` under the hood to implement this allocator. I can't just use `Vec` directly as it exists today because
- If I'm the global allocator, then using it will just introduce a recursive dependency on myself
- If I'm in a no-std environment, there is no `Vec` as it exists today

Thus, what I need is to be able to use a `Vec` that is parametrized on another allocator that I use internally for simple internal bookkeeping. This is the goal of [bsalloc](https://crates.io/crates/bsalloc) (and the source of the name - it is used to bootstrap other allocators).

In elfmalloc, we are still able to be a global allocator by:
- When compiling ourselves, statically compile jemalloc as the global allocator
- Produce a shared object file that can be dynamically loaded by other programs

Note that in this case, it's important that we don't compile ourselves using the system allocator as the global allocator because then, once loaded, we'd re-introduce the recursive dependency because, at that point, we *are* the system allocator.

But it doesn't work when:
- Somebody wants to use us as the global allocator in Rust in the "official" way (as opposed to by creating a shared object file first)
- We're in a no-std environment

----

> OTOH, the C++ standard does not support over-aligned types like SIMD types, and if you try to heap allocate one with new you will invoke undefined behavior (you need to use `posix_memalign` or similar).

Since our current `Alloc` trait takes alignment as a parameter, I assume this class of problem (the "I can't work without a different alignment" problem) goes away for us?

---

### Comment 272 by raphaelcohn
_2018-01-19T11:32:13Z_

@gnzlbg - a comprehensive write-up (thank you) but none of the use cases cover persistent memory\*.

This use case ***must*** be considered. In particular, it strongly influences what the right thing to do is:-
* That more than one allocator is in use, and especially, when used that allocator is for persistent memory, it would **never be the system allocator**; (indeed, there could be *multiple* persistent memory allocators)
* The cost of 're-implementing' the standard collections is high, and leads to incompatible code with third-party libraries.
* The allocator's lifetime is not necessarily `'static`.
* That objects stored in persistent memory need additional state that must be populated from the heap, ie they need state reininitialized. This is particularly so for mutexes and the like. What was once disposable no longer is disposed.

Rust has a superb opportunity to seize the initiative here, and make it a first-class platform for what will replace HDs, SSDs and even PCI-attached storage.

\*Not a surprise, really, because until very recently it's been a bit special. It's now widely supported in Linux, FreeBSD and Windows.

---

### Comment 273 by rpjohnst
_2018-01-19T17:54:39Z_

@raphaelcohn 

This really isn't the place to work out persistent memory. Yours is not the only school of thought regarding the interface to persistent memory- for instance, it may turn out that the prevailing approach is simply to treat it like faster disk, for data integrity reasons.

If you have a use case for using persistent memory this way it would probably be better to make that case elsewhere somehow first. Prototype it, come up with some more concrete changes to the allocator interface, and ideally make the case that those changes are worth the impact they would have on the average case.

---

### Comment 274 by raphaelcohn
_2018-01-20T18:05:47Z_

@rpjohnst 

I disagree. This is exactly the sort of place it belongs. I want to avoid a decision being made that creates a design that is the result of too narrow a focus and search for evidence.

The current Intel PMDK - which is where a lot of effort for low-level user space support is focused - approaches it far more as allocated, regular memory with pointers - memory that is similar to that via `mmap`, say. Indeed, if one wants to work with persistent memory on Linux, then, I believe it's pretty much your only port of call at the moment. In essence, one of the most advanced toolkits for using it - the prevailing one if you will - treats it as allocated memory.

As for prototyping it - well, that's exactly what I said I have done:-

> I've been working recently on a Rust wrapper for a persistent memory allocator (specifically, libpmemcto).

(You can use an early-days version of my crate at <https://crates.io/crates/nvml>. There's a lot more experimentation in source control in the `cto_pool` module).

My prototype is built in mind with what is needed to replace a data storage engine in a real-world, large scale system. A similar mindset is behind many of my open source projects. I've found over many years the best libraries, as are the best standards, ones which derive _from_ real usage.

Nothing like trying to fit a real-world allocator to the current interface. Frankly, the experience of using the `Alloc` interface, then copying the whole of `Vec`, then tweaking it, was painful. A lot of places assume that allocators aren't passed in, eg `Vec::new()`.

In doing it, I made some observations in my original comment about what would be required of an allocator, and what would be required of an user of such an allocator. I think those are very valid on a discussion thread about an allocator interface.

---

### Comment 275 by Ericson2314
_2018-01-21T16:36:21Z_

The good news is your first 3 bullet points from https://github.com/rust-lang/rust/issues/32838#issuecomment-358940992 are shared by other use-cases.

---

### Comment 276 by gnzlbg
_2018-01-21T17:34:02Z_

I just wanted to add that i did not add non-volatile memory to the list
because the list listed use cases of allocators parametrizing containers in
the C++ world that are “widely” used, at least on my experience (those
alloctors I mentioned are mostly from very popular libraries used by many).
While I know of the efforts of the Intel SDK (some of their libraries
target C++) I don’t personally know any projects using them (do they have
an allocator that can be used with std::vector? I don’t know). This doesn’t
mean that they aren’t used nor important. I’d be interested into knowing
about these, but the main point of my post was that parametrizing
allocators by containers is very complex, and we should try to make
progress with system allocators without closing any doors for containers
(but we should tackle that later).

On Sun 21. Jan 2018 at 17:36, John Ericson <notifications@github.com> wrote:

> The good news is your first 3 bullet points from #32838 (comment)
> <https://github.com/rust-lang/rust/issues/32838#issuecomment-358940992>
> are shared by other use-cases.
>
> —
> You are receiving this because you were mentioned.
> Reply to this email directly, view it on GitHub
> <https://github.com/rust-lang/rust/issues/32838#issuecomment-359261305>,
> or mute the thread
> <https://github.com/notifications/unsubscribe-auth/AA3Npk95PZBZcm7tknNp_Cqrs_3T1UkEks5tM2ekgaJpZM4IDYUN>
> .
>


---

### Comment 277 by emoon
_2018-01-31T18:44:23Z_

I tried to read most of what has been written already so this may be here already and in that case I'm sorry if I missed it but here goes:

Something that is fairly common for games (in C/C++) is is to use "per frame scratch allocation" What this means is there is an linear/bump allocator that is used for allocations that are alive for a certain period of time (in a game frame) and then "destroyed".

Destroyed in this case meaning that you reset the allocator back to it's starting position. There is no "destruction" of objects at all as these objects has to be of POD type (thus no destructors are being executed)

I wonder if something like this will fit with the current allocator design in Rust? 
 
(edit: It should be there is NO destruction of objects)



---

### Comment 278 by joshlf
_2018-01-31T19:10:11Z_

@emoon 

> Something that is fairly common for games (in C/C++) is is to use "per frame scratch allocation" What this means is there is an linear/bump allocator that is used for allocations that are alive for a certain period of time (in a game frame) and then "destroyed".
> 
> Destroyed in this case meaning that you reset the allocator back to it's starting position. There is "destruction" of objects at all as these objects has to be of POD type (thus no destructors are being executed)

Should be doable. Off the top of my head, you'd need one object for the arena itself and another object that is a per-frame handle on the arena. Then, you could implement `Alloc` for that handle, and assuming you were using high-level safe wrappers for allocation (e.g., imagine that `Box` becomes parametric on `Alloc`), the lifetimes would ensure that all of the allocated objects were dropped before the per-frame handle was dropped. Note that `dealloc` would still be called for each object, but if `dealloc` was a no-op, then the entire drop-and-deallocate logic might be completely or mostly optimized away.

---

### Comment 279 by rpjohnst
_2018-01-31T20:51:03Z_

You could also use a custom smart pointer type that doesn't implement `Drop`, which would make a lot of things easier elsewhere.

---

### Comment 280 by emoon
_2018-02-01T10:07:18Z_

Thanks! I made a typo in my original post. It's to say that there is **no** destruction of objects.

---

### Comment 281 by alexreg
_2018-02-09T18:29:17Z_

For people who aren't expert in allocators, and can't follow this thread, what's the current consensus: do we plan to support custom allocators for the stdlib collection types?

---

### Comment 282 by Ericson2314
_2018-02-09T22:59:31Z_

@alexreg I'm not sure what the final plan is, but there is confirmed 0 technical difficulties in doing so. OTOH we don't have a good way to then expose that in `std` because default type variables are suspect, but I have no problem with just making it an `alloc`-only thing for now so we can make progress on the lib side unimpeded.

---

### Comment 283 by alexreg
_2018-02-10T00:42:01Z_

@Ericson2314 Okay, good to hear. Are default type variables implemented yet? Or at RFC stage perhaps? As you say, if they're just restricted to things related to alloc / `std::heap`, it should all be okay.

---

### Comment 284 by cramertj
_2018-02-10T01:13:16Z_

@alexreg See https://github.com/rust-lang/rfcs/pull/2321

---

### Comment 285 by brunoczim
_2018-02-20T18:35:19Z_

I really think AllocErr should be Error. It would be more consistent with another modules (e.g. io).

---

### Comment 286 by SimonSapin
_2018-02-20T20:43:48Z_

`impl Error for AllocError` probably makes sense and doesn’t hurt, but I’ve personally found the `Error` trait to be useless.

---

### Comment 287 by glandium
_2018-02-25T21:53:47Z_

I was looking at at the Layout::from_size_align function today, and the "`align` must not exceed 2^31 (i.e. `1 << 31`)," limitation did not make sense to me. And git blame pointed to #30170.

I must say that was a quite deceptive commit message there, talking about `align` fitting in a u32, which is only incidental, when the actual thing being "fixed" (more worked around) is a system allocator misbehaving.

Which leads me to this note: The "OSX/alloc_system is buggy on huge alignments" item here should *not* be checked. While the direct issue has been dealt with, I don't think the fix is right for the long term: Because a system allocator misbehaves shouldn't prevent *implementing* an allocator that behaves. And the arbitrary limitation on Layout::from_size_align does that.

---

### Comment 288 by SimonSapin
_2018-02-25T22:00:06Z_

@glandium Is it useful to request alignment to a multiple of 4 gigbytes or more?

---

### Comment 289 by glandium
_2018-02-25T22:10:32Z_

I can imagine cases where one may want to have an allocation of 4GiB aligned at 4GiB, which is not possible currently, but hardly more. But I don't think arbitrary limitations should be added just because we don't think of such reasons now.

---

### Comment 290 by sfackler
_2018-02-25T22:57:27Z_

> I can imagine cases where one may want to have an allocation of 4GiB aligned at 4GiB

What are those cases?

---

### Comment 291 by joshlf
_2018-02-25T23:11:53Z_

> > I can imagine cases where one may want to have an allocation of 4GiB aligned at 4GiB
>
> What are those cases?

Concretely, I just added support for arbitrarily large alignments in [`mmap-alloc`](https://crates.io/crates/mmap-alloc) in order to support allocating large, aligned slabs of memory for use in [`elfmalloc`](https://crates.io/crates/elfmalloc). The idea is to have the slab of memory be aligned to its size so that, given a pointer to an object allocated from that slab, you merely need to mask off the low bits to find the containing slab. We don't currently use slabs that are 4GB in size (for objects that large, we go directly to mmap), but there's no reason that we couldn't, and I could totally imagine an application with large RAM requirements that wanted to do that (that is, if it allocated multi-GB objects frequently enough that it didn't want to accept the overhead of mmap).

---

### Comment 292 by scottlamb
_2018-02-26T05:18:31Z_

Here's a possible use case for > 4GiB alignment: alignment to a large page boundary. There already are platforms that support > 4 GiB pages. [This IBM document](https://www.ibm.com/support/knowledgecenter/en/ssw_aix_61/com.ibm.aix.performance/multiple_page_size_support.htm) say "the POWER5+ processor supports four virtual memory page sizes: 4 KB, 64 KB, 16 MB, and 16 GB." Even x86-64 isn't far off: "huge pages" typically are 2 MiB, but it also [supports](https://wiki.debian.org/Hugepages#x86_64) 1 GiB.

---

### Comment 293 by glandium
_2018-03-04T11:56:28Z_

All the non-typed functions in the Alloc trait are dealing with `*mut u8`. Which means they could take or return null pointers, and all hell would break loose. Should they use `NonNull` instead?

---

### Comment 294 by sfackler
_2018-03-04T17:53:56Z_

There are many pointers that they could return from which all hell would
break loose.
On Sun, Mar 4, 2018 at 3:56 AM Mike Hommey <notifications@github.com> wrote:

> All the non-typed functions in the Alloc trait are dealing with *mut u8.
> Which means they could take or return null pointers, and all hell would
> break loose. Should they use NonNull instead?
>
> —
> You are receiving this because you were mentioned.
> Reply to this email directly, view it on GitHub
> <https://github.com/rust-lang/rust/issues/32838#issuecomment-370223269>,
> or mute the thread
> <https://github.com/notifications/unsubscribe-auth/ABY2UR2dRxDtdACeRUh_djM-DExRuLxiks5ta9aFgaJpZM4IDYUN>
> .
>


---

### Comment 295 by joshlf
_2018-03-04T21:12:42Z_

A more compelling reason to use `NonNull` is that it would allow the `Result`s currently returned from `Alloc` methods (or `Options`, if we switch to that in the future) to be smaller.

---

### Comment 296 by glandium
_2018-03-04T22:07:11Z_

> A more compelling reason to use NonNull is that it would allow the Results currently returned from Alloc methods (or Options, if we switch to that in the future) to be smaller.

I don't think it would because `AllocErr` has two variants.

> There are many pointers that they could return from which all hell would break loose.

But a null pointer is clearly more wrong than any other pointer.

I like to think that the rust type system helps with footguns, and is used to encode invariants. The documentation for `alloc` clearly says "If this method returns an `Ok(addr)`, then the addr returned will be non-null address", but its return type doesn't. As things are, `Ok(malloc(layout.size()))` would be a valid implementation, when it clearly isn't.

Note, there are also notes about `Layout` size needing to be non-zero, so I'd also argue it should encode that as a NonZero<usize>.

It's not because all those functions are inherently unsafe that we shouldn't have some footgun prevention.

---

### Comment 297 by hanna-kruppe
_2018-03-04T22:24:01Z_

Out of all the possible errors in using (edit: and implementing) allocators, passing a null pointer is one of the easiest to track down (you always get a clean segfault on dereference, at least if you have an MMU and didn't do very weird things with it), and usually one of the most trivial ones to fix as well. It's true that unsafe interfaces can try to prevent footguns, but this footgun seems disproportionally small (compared to the other possible errors, and to the verbosity of encoding this invariant in the type system).

Besides, it seems likely that allocator implementations would just use the unchecked constructor of `NonNull` "for performance": since in a correct allocator would never return null anyway, it would want to skip the `NonNell::new(...).unwrap()`. In that case you won't actually get any tangible footgun prevention, just more boilerplate. (The `Result` size benefits, if real, may still be a compelling reason for it.)

---

### Comment 298 by SimonSapin
_2018-03-04T22:55:26Z_

> allocator implementations would just use the unchecked constructor of NonNull

The point is less to help allocator implementation than to help their users. If `MyVec` contains a `NonNull<T>` and `Heap.alloc()` already returns a `NonNull`, that one less checked or unsafe-unchecked call that I need to make.

---

### Comment 299 by glandium
_2018-03-04T23:30:52Z_

Note that pointers are not only return types, they are also input types to e.g. `dealloc` and `realloc`. Are those functions supposed to harden against their input being possibly null or not? The documentation would tend to say no, but the type system would tend to say yes.

Quite similarly with layout.size(). Are allocation functions supposed to handle the requested size being 0 somehow, or not?

> (The Result size benefits, if real, may still be a compelling reason for it.)

I doubt there are size benefits, but with something like #48741, there would be codegen benefits.

---

### Comment 300 by SimonSapin
_2018-03-05T06:59:37Z_

If we continue that principle of being more flexible for users of the API, pointers should be `NonNull` in return types but not in arguments. (This doesn’t mean that those arguments should be null-checked at runtime.)

---

### Comment 301 by joshlf
_2018-03-05T10:05:18Z_

I think a Postel's law approach is the wrong one to take here. Is there any
case in which passing a null pointer to an Alloc method is valid? If not,
then that flexibility is basically just giving the footgun a slightly more
sensitive trigger.

On Mar 5, 2018 8:00 AM, "Simon Sapin" <notifications@github.com> wrote:

> If we continue that principle of being more flexible for users of the API,
> pointers should be NonNull in return types but not in arguments. (This
> doesn’t mean that those arguments should be null-checked at runtime.)
>
> —
> You are receiving this because you are subscribed to this thread.
> Reply to this email directly, view it on GitHub
> <https://github.com/rust-lang/rust/issues/32838#issuecomment-370327018>,
> or mute the thread
> <https://github.com/notifications/unsubscribe-auth/AA_2L8zrOLyUv5mUc_kiiXOAn1f60k9Uks5tbOJ0gaJpZM4IDYUN>
> .
>


---

### Comment 302 by hanna-kruppe
_2018-03-05T11:53:39Z_

> The point is less to help allocator implementation than to help their users. If MyVec contains a NonNull<T> and Heap.alloc() already returns a NonNull, that one less checked or unsafe-unchecked call that I need to make.

Ah this makes sense. Doesn't fix the footgun, but centralizes the responsibility for it.

> Note that pointers are not only return types, they are also input types to e.g. dealloc and realloc. Are those functions supposed to harden against their input being possibly null or not? The documentation would tend to say no, but the type system would tend to say yes.

> Is there any case in which passing a null pointer to an Alloc method is valid? If not, then that flexibility is basically just giving the footgun a slightly more sensitive trigger.

The user absolutely *has* to read the documentation and keep the invariants in mind. Many invariants can't be enforced via type system at all -- if they could, the function wouldn't be unsafe to begin with. So this is solely a question of whether putting NonNull in any given interface will actually help users by

- reminding them to read the docs and think about the invariants
- offering convenience (@SimonSapin's point wrt alloc's return value)
- giving some material advantage (e.g., layout optimizations)

I don't see any strong advantage in making e.g., the argument of `dealloc` into `NonNull`. I see roughly two classes of uses of this API:

1. Relatively trivial use, where you call `alloc`, store the returned pointer somewhere, and after a while pass the stored pointer to `dealloc`.
2. Complicated scenarios involving FFI, lots of pointer arithmetic, etc. where there's significant logic involved in ensuring you pass the right thing to `dealloc` at the end.

Taking `NonNull` here basically only helps the first kind of use case, because those will store the `NonNull` in some nice place and just pass it to `NonNull` unaltered. Theoretically it could prevent some typos (passing `foo` when you meant `bar`) if you're juggling multiple pointers and only one of them is `NonNull`, but this doesn't seem too common or important. The disadvantage of `dealloc` taking a raw pointer (assuming `alloc` returns `NonNull` which @SimonSapin has convinced me should happen) would be that it requires an `as_ptr` in the dealloc call, which is potentially annoying but doesn't impact safety either way.

The second kind of use case is not helped because it likely can't keep using `NonNull` throughout the whole process, so it would have to manually re-create a `NonNull` from the raw pointer it got by whatever means. As I argued earlier, this would likely become an unchecked/`unsafe` assertion rather than an actual run time check, so no footguns are prevented.

This is not to say I am in favor of `dealloc` taking a raw pointer. I just don't see any the claimed advantages wrt footguns. Consistency of the types probably just wins by default.

---

### Comment 303 by Ericson2314
_2018-03-05T20:59:10Z_

I'm sorry but I read this as "Many invariants can't be enforced via type system at all...therefore let's not even try". Don't let the perfect be the enemy of the good!

---

### Comment 304 by cramertj
_2018-03-05T21:06:16Z_

I think it's more about the tradeoffs between the guarantees provided by `NonNull` and the ergonomics lost from having to transition back and forth between `NonNull` and raw pointers. I don't have a particularly strong opinion either way-- neither side seems unreasonable.

---

### Comment 305 by Ericson2314
_2018-03-05T21:14:36Z_

@cramertj Yeah, but I don't really buy the premise of that sort of argument. People say `Alloc` is for obscure, hidden, and largely unsafe use-cases. Well, in obscure, hard-to-read code, I would like to have as much safety as possible---precisely because they are so rarely touched its likely the original author won't be around. Conversely, if the code is being read years later, screw egonomics. If anything, it is counter-productive. The code should strive to be very explicit so an unfamiliar reader can better figure out what on earth is going on. Less noise < clearer invariants.

---

### Comment 306 by Ericson2314
_2018-03-05T21:17:13Z_

> The second kind of use case is not helped because it likely can't keep using `NonNull` throughout the whole process, so it would have to manually re-create a `NonNull` from the raw pointer it got by whatever means.

This is simply a coordination failure, not a technical inevitability. Sure, right now many unsafe APIs might use raw pointers. So *something* has to lead the way switching to a superior interface using `NonNull` or other wrappers. Then other code can more easily follow suit. I see 0 reason to constantly fall back on hard-to-read, uninformative raw-pointers in greenfield, all-Rust, unsafe code.

---

### Comment 307 by fitzgen
_2018-03-05T21:24:35Z_

Hi!

I just want to say that, as the author/maintainer of a Rust custom allocator, I am in favor of `NonNull`. Pretty much for all the reasons that have already been laid out in this thread.

Also, I'd like to point out that @glandium is the maintainer of firefox's fork of jemalloc, and has lots of experience hacking on allocators as well.

---

### Comment 308 by hanna-kruppe
_2018-03-05T21:30:22Z_

I could write much much more responding to @Ericson2314 and others, but it's quickly becoming a very detached and philosophical debate, so I'm cutting it short here. I was arguing against what I believe is an over-statement of the *safety* benefits of `NonNull` in this sort of API (there are other benefits, of course). That is not to say there are no safety benefits, but as @cramertj said, there are trade offs and I think the "pro" side is overstated. Regardless, I have already said I lean towards using `NonNull` in various places for other reasons -- for the reason @SimonSapin gave in `alloc`, in `dealloc` for consistency. So let's do that and not go off on any more tangents.

---

### Comment 309 by Ericson2314
_2018-03-05T22:47:19Z_

If there's a few `NonNull` use-cases that everyone is on board with, that's a great start.

---

### Comment 310 by gnzlbg
_2018-03-06T11:48:09Z_

We'll probably want to update `Unique` and friends to use `NonNull` instead of `NonZero` to keep friction at least within `liballoc` low. But this does look like something we are already doing anyways at one abstraction level over the allocator, and I can't think of any reasons for not doing this at the allocator level as well. Looks like a reasonable change to me.

---

### Comment 311 by SimonSapin
_2018-03-06T12:50:31Z_

(There’s already two implementations of the `From` trait that convert safely between `Unique<T>` and `NonNull<T>`.)

---

### Comment 312 by glandium
_2018-03-28T07:46:27Z_

Considering I need something very much like the allocator API in stable rust, I extracted the code from the rust repo and put it in a separate crate:
- https://crates.io/crates/allocator_api
- https://github.com/glandium/allocator_api

This /could/ be used to iterate on experimental changes to the API, but as of now, it's a plain copy of what is in the rust repo.

---

### Comment 313 by Ericson2314
_2018-03-28T18:06:00Z_

[Off topic] Yeah it would be nice if `std` could use out of tree stable code crates like that, so that we can experiment with unstable interfaces in stable code. This is one of the reasons I like having `std` a facade.

---

### Comment 314 by SimonSapin
_2018-03-28T18:24:56Z_

`std` could depend on a copy of a crate from crates.io, but if your program also depends on the same crate it wouldn’t "look like" the same crate/types/traits to rustc anyway so I don’t see how it would help. Anyway, regardless of the facade, making unstable features unavailable on the stable channel is a very deliberate choice, not an accident.

---

### Comment 315 by glandium
_2018-03-29T02:09:36Z_

It seems we have some agreement wrt using NonNull. What's the way forward for this to actually happen? just a PR doing it? a RFC?

Unrelatedly, I've been looking at some assembly generated from Boxing things, and the error paths are rather large. Should something be done about it?

Examples:
```rust
pub fn bar() -> Box<[u8]> {
    vec![0; 42].into_boxed_slice()
}

pub fn qux() -> Box<[u8]> {
    Box::new([0; 42])
}
```
compiles to:
```
example::bar:
  sub rsp, 56
  lea rdx, [rsp + 8]
  mov edi, 42
  mov esi, 1
  call __rust_alloc_zeroed@PLT
  test rax, rax
  je .LBB1_1
  mov edx, 42
  add rsp, 56
  ret
.LBB1_1:
  mov rax, qword ptr [rsp + 8]
  movups xmm0, xmmword ptr [rsp + 16]
  movaps xmmword ptr [rsp + 32], xmm0
  mov qword ptr [rsp + 8], rax
  movaps xmm0, xmmword ptr [rsp + 32]
  movups xmmword ptr [rsp + 16], xmm0
  lea rdi, [rsp + 8]
  call __rust_oom@PLT
  ud2

example::qux:
  sub rsp, 104
  xorps xmm0, xmm0
  movups xmmword ptr [rsp + 58], xmm0
  movaps xmmword ptr [rsp + 48], xmm0
  movaps xmmword ptr [rsp + 32], xmm0
  lea rdx, [rsp + 8]
  mov edi, 42
  mov esi, 1
  call __rust_alloc@PLT
  test rax, rax
  je .LBB2_1
  movups xmm0, xmmword ptr [rsp + 58]
  movups xmmword ptr [rax + 26], xmm0
  movaps xmm0, xmmword ptr [rsp + 32]
  movaps xmm1, xmmword ptr [rsp + 48]
  movups xmmword ptr [rax + 16], xmm1
  movups xmmword ptr [rax], xmm0
  mov edx, 42
  add rsp, 104
  ret
.LBB2_1:
  movups xmm0, xmmword ptr [rsp + 16]
  movaps xmmword ptr [rsp + 80], xmm0
  movaps xmm0, xmmword ptr [rsp + 80]
  movups xmmword ptr [rsp + 16], xmm0
  lea rdi, [rsp + 8]
  call __rust_oom@PLT
  ud2
```

That's a rather large amount of code to add to any place creating boxes. Compare with 1.19, which didn't have the allocator api:
```
example::bar:
  push rax
  mov edi, 42
  mov esi, 1
  call __rust_allocate_zeroed@PLT
  test rax, rax
  je .LBB1_2
  mov edx, 42
  pop rcx
  ret
.LBB1_2:
  call alloc::oom::oom@PLT

example::qux:
  sub rsp, 56
  xorps xmm0, xmm0
  movups xmmword ptr [rsp + 26], xmm0
  movaps xmmword ptr [rsp + 16], xmm0
  movaps xmmword ptr [rsp], xmm0
  mov edi, 42
  mov esi, 1
  call __rust_allocate@PLT
  test rax, rax
  je .LBB2_2
  movups xmm0, xmmword ptr [rsp + 26]
  movups xmmword ptr [rax + 26], xmm0
  movaps xmm0, xmmword ptr [rsp]
  movaps xmm1, xmmword ptr [rsp + 16]
  movups xmmword ptr [rax + 16], xmm1
  movups xmmword ptr [rax], xmm0
  mov edx, 42
  add rsp, 56
  ret
.LBB2_2:
  call alloc::oom::oom@PLT
```

---

### Comment 316 by tomaka
_2018-03-29T13:19:51Z_

If this is actually significant, then it's indeed annoying. However maybe LLVM optimizes this out for larger programs?

---

### Comment 317 by glandium
_2018-03-29T13:57:15Z_

There are 1439 calls to `__rust_oom` in latest Firefox nightly. Firefox doesn't use rust's allocator, though, so we get direct calls to malloc/calloc, followed by a null check that the jumps to the oom preparation code, which is usually two movq and a lea, filling the AllocErr and getting its address to pass it to `__rust__oom`. That's the best case scenario, essentially, but that's still 20 bytes of machine code for the two movq and the lea.

It I look at ripgrep, there are 85, and they are all in identical `_ZN61_$LT$alloc..heap..Heap$u20$as$u20$alloc..allocator..Alloc$GT$3oom17h53c76bda5
0c6b65aE.llvm.nnnnnnnnnnnnnnn` functions. All of them are 16 bytes long. There are 685 calls to those wrapper functions, most of which are preceded by code similar to what I pasted in https://github.com/rust-lang/rust/issues/32838#issuecomment-377097485 .

---

### Comment 318 by emilio
_2018-03-29T15:17:20Z_

@nox was looking today at enabling the `mergefunc` llvm pass, I wonder if that makes any difference here.

---

### Comment 319 by glandium
_2018-03-29T21:15:30Z_

`mergefunc` apparently doesn't get rid of the multiple identical `_ZN61_$LT$alloc..heap..Heap$u20$as$u20$alloc..allocator..Alloc$GT$3oom17h53c76bda5 0c6b65aE.llvm.nnnnnnnnnnnnnnn` functions (tried with `-C passes=mergefunc` in `RUSTFLAGS`).

But what makes a big difference is LTO, which is actualy what makes Firefox call malloc directly, leaving the creation of the `AllocErr` to right before calling `__rust_oom`. That also makes the creation of the `Layout` unnecessary before calling the allocator, leaving it to when filling the `AllocErr`.

This makes me think the allocation functions, except `__rust_oom` should probably be marked inline.

BTW, having looked at the generated code for Firefox, I'm thinking it would ideally be desirable to use `moz_xmalloc` instead of `malloc`. This is not possible without a combination of the Allocator traits and being able to replace the global heap allocator, but brings the possible need for a custom error type for the Allocator trait: `moz_xmalloc` is infallible and never returns in case of failure. IOW, it handles OOM itself, and the rust code wouldn't need to call `__rust_oom` in that case. Which would make it desirable for the allocator functions to optionally return `!` instead of `AllocErr`.

---

### Comment 320 by SimonSapin
_2018-03-29T23:40:43Z_

We’ve discussed making `AllocErr` a zero-size struct, which might also help here. With the pointer also made `NonNull`, the entire return value could be pointer-sized.

---

### Comment 321 by SimonSapin
_2018-04-04T21:45:50Z_

https://github.com/rust-lang/rust/pull/49669 makes a number of changes to these APIs, with the goal of stabilizing a subset covering global allocators. Tracking issue for that subset: https://github.com/rust-lang/rust/issues/49668. In particular, a **new `GlobalAlloc` trait** is introduced.

---

### Comment 322 by alexreg
_2018-04-04T22:10:35Z_

Will this PR allow us to do things like `Vec::new_with_alloc(alloc)` where `alloc: Alloc` soon?

---

### Comment 323 by sfackler
_2018-04-04T22:13:17Z_

@alexreg no

---

### Comment 324 by alexreg
_2018-04-04T22:24:44Z_

@sfackler Hmm, why not? What do we need before we can do that? I don't really get the point of this PR otherwise, unless it's for simply changing the global allocator.

---

### Comment 325 by cramertj
_2018-04-04T22:37:15Z_

@alexreg 
> I don't really get the point of this PR otherwise, unless it's for simply changing the global allocator.

I think it's simply for changing the global allocator.

---

### Comment 326 by SimonSapin
_2018-04-04T22:49:35Z_

@alexreg If you mean on stable, there are a number of unresolved design question that we’re not ready to stabilize. On Nightly, this is [supported by `RawVec`](https://doc.rust-lang.org/nightly/alloc/raw_vec/struct.RawVec.html#method.new_in) and probably fine to add as `#[unstable]` for `Vec` for anyone who feels like working on that.

And yes, as mentioned in the PR its point is to allow changing the global allocator, or allocating (e.g. in a custom collection type) without absuing `Vec::with_capacity`.

---

### Comment 327 by glandium
_2018-04-04T23:00:00Z_

FWIW, the `allocator_api` crate mentioned in https://github.com/rust-lang/rust/issues/32838#issuecomment-376793369 has `RawVec<T, A>` and `Box<T, A>` on the master branch (not released yet). I'm thinking of it as an incubator for what collections generic over the allocation type could look like (plus the fact that I do need a `Box<T, A>` type for stable rust). I haven't started porting vec.rs to add `Vec<T, A>` just yet, but PRs are welcome. vec.rs is large.

---

### Comment 328 by glandium
_2018-04-04T23:03:22Z_

I'll note that the codegen "issues" mentioned in https://github.com/rust-lang/rust/issues/32838#issuecomment-377097485 should be gone with the changes in #49669.

Now, with some more thought given to using the `Alloc` trait to help implement an allocator in layers, there are two things that I think would be useful (at least to me):
- as mentioned earlier, optionally being able to specify a different `AllocErr` type. This can be useful to make it `!`, or, now that AllocErr is empty, to optionally have it convey more information than "failed".
- optionally being able to specify a different `Layout` type. Imagine you have two layers of allocators: one for page allocations, and one for larger regions. The latter can rely on the former, but if they both take the same `Layout` type, then both layers need to do their own validation: at the lowest level, that size and alignment is a multiple of the page size, and the higher level, that size and alignment match the requirements for the larger regions. But those checks are redundant. With specialized `Layout` types, the validation could be delegated to the `Layout` creation instead of in the allocator itself, and conversions between the `Layout` types would allow to skip the redundant checks.

---

### Comment 329 by alexreg
_2018-04-04T23:05:09Z_

@cramertj @SimonSapin @glandium Okay, thanks for clarifying. I may just submit a PR for some of the other collections-prime types. Is it best to do this against your allocator-api repo/crate, @glandium, or rust master?

---

### Comment 330 by glandium
_2018-04-04T23:08:03Z_

@alexreg considering the amount of breaking changes to the `Alloc` trait in #49669, it's probably better to wait for it to merge first.

---

### Comment 331 by alexreg
_2018-04-04T23:47:18Z_

@glandium Fair enough. That doesn't seem *too* far away from landing. I just noticed the https://github.com/pnkfelix/collections-prime repo too... what's that in relation to yours?

---

### Comment 332 by Amanieu
_2018-04-04T23:57:27Z_

I would add one more open question:

- Is `Alloc::oom` allowed to panic? Currently the docs say that this method must abort the process. This has implications for code that uses allocators since they must then be designed to handle unwinding properly without leaking memory.

I think that we should allow panicking since a failure in a local allocator does not necessarily mean that the global allocator will fail as well. In the worst case, the global allocator's `oom` will be called which will abort the process (doing otherwise would break existing code).

---

### Comment 333 by glandium
_2018-04-04T23:58:16Z_

@alexreg It's not. It just seems to be a plain copy of what's in std/alloc/collections. Well, a two-year old copy of it. My crate is much more limited in scope (the published version only has the `Alloc` trait as of a few weeks ago, the master branch only has `RawVec` and `Box` on top of that), and one of my goals is to keep it building with stable rust.

---

### Comment 334 by alexreg
_2018-04-05T00:55:12Z_

@glandium Okay, in that case it probably makes sense for me to wait until that PR lands, then create a PR against rust master and tag you, so you know when it gets merged into master (and can then merge it into your crate), fair?

---

### Comment 335 by glandium
_2018-04-05T00:59:20Z_

@alexreg makes sense. You /could/ start working on it now, but that would likely induce some churn on your end if/when bikeshedding changes things in that PR.

---

### Comment 336 by alexreg
_2018-04-05T01:30:51Z_

@glandium I've got other things to keep me busy with Rust for now, but I'll be onto it when that PR gets approved. It will be great go get allocator-generic heap allocation / collections on both nightly and stable soon. :-)

---

### Comment 337 by gnzlbg
_2018-04-05T06:39:22Z_

> Is Alloc::oom allowed to panic? Currently the docs say that this method must abort the process. This has implications for code that uses allocators since they must then be designed to handle unwinding properly without leaking memory.

@Amanieu This RFC was merged: https://github.com/rust-lang/rfcs/pull/2116 The docs and implementation might just not have been updated yet.

---

### Comment 338 by glandium
_2018-05-03T21:53:45Z_

There is one change to the API that I'm considering to submit a PR for:

Split the `Alloc` trait in two parts: "implementation" and "helpers". The former would be functions like `alloc`, `dealloc`, `realloc`, etc. and the latter, `alloc_one`, `dealloc_one`, `alloc_array`, etc. While there are some hypothetical benefits from being able to have custom implementation for the latter, it's far from the most common need, and when you need to implement generic wrappers (which I've found to be incredibly common, to the point I've actually started to write a custom derive for that), you still need to implement all of them because the wrappee might be customizing them.

OTOH, if an `Alloc` trait implementer does try to do fancy things in e.g. `alloc_one`, they're not guaranteed that `dealloc_one` will be called for that allocation. There are multiple reasons for this:

  - The helpers are not used consistently. Just one example, `raw_vec` uses a mix of `alloc_array`, `alloc`/`alloc_zeroed`, but only uses `dealloc`.
  - Even with consistent use of e.g. `alloc_array`/`dealloc_array`, one can still safely convert a `Vec` into a `Box`, which would then use `dealloc`.
  - Then there are some parts of the API that just don't exist (no zeroed version of `alloc_one`/`alloc_array`)

So even though there are actual use cases for specialization of e.g. `alloc_one` (and as a matter of fact, I do have such a need for mozjemalloc), one is better off using a specialized allocator instead.

Actually, it's worse than that, in the rust repo, there's exactly one use of `alloc_array`, and no use of `alloc_one`, `dealloc_one`, `realloc_array`, `dealloc_array`. Not even box syntax uses `alloc_one`, it uses `exchange_malloc`, which takes a `size` and `align`. So those functions are more meant as a convenience for clients than for implementers.

With something like `impl<A: Alloc> AllocHelpers for A` (or `AllocExt`, whatever name is chosen), we'd still have the convenience of those functions for clients, while not allowing implementers to shoot themselves in the foot if they thought they'd do fancy things by overriding them (and making it easier on people implementing proxy allocators).

---

### Comment 339 by glandium
_2018-05-03T23:41:55Z_

> There is one change to the API that I'm considering to submit a PR for

Did so in #50436

---

### Comment 340 by gnzlbg
_2018-05-04T09:44:08Z_

@glandium 

> (and as a matter of fact, I do have such a need for mozjemalloc),

Could you elaborate on this use case?

---

### Comment 341 by glandium
_2018-05-04T09:48:56Z_

mozjemalloc has a base allocator that purposefully leaks. Except for one kind of objects, where it keeps a free list. I can do that by layering allocators rather than do tricks with `alloc_one`.

---

### Comment 342 by retep998
_2018-05-07T20:27:05Z_

>Is it required to deallocate with the exact alignment that you allocated with?

Just to reinforce that the answer to this question is **YES**, I have this lovely quote from [Microsoft themselves](https://blogs.msdn.microsoft.com/vcblog/2018/05/07/announcing-msvc-conforms-to-the-c-standard/):

>aligned_alloc() will probably never be implemented, as C11 specified it in a way that’s incompatible with our implementation (namely, that free() must be able to handle highly aligned allocations)

Using the system allocator on Windows will *always* require knowing the alignment when deallocating in order to correctly deallocate highly aligned allocations, so can we please just mark that question as resolved?

---

### Comment 343 by ruuda
_2018-05-07T21:32:42Z_

> Using the system allocator on Windows will *always* require knowing the alignment when deallocating in order to correctly deallocate highly aligned allocations, so can we please just mark that question as resolved?

It’s a shame, but it is the way it is. Let’s give up on overaligned vectors then. :confused:

---

### Comment 344 by gnzlbg
_2018-05-07T21:35:19Z_

> Let’s give up on overaligned vectors then

How come? You just need `Vec<T, OverAlignedAlloc<U16>>` that both allocates and deallocates with overalignment.

---

### Comment 345 by ruuda
_2018-05-07T22:01:19Z_

> How come? You just need `Vec<T, OverAlignedAlloc<U16>>` that both allocates and deallocates with overalignment.

I should have been more specific. I meant moving overaligned vectors into an API outside of your control, i.e. one that takes a `Vec<T>` and not `Vec<T, OverAlignedAlloc<U16>>`. (For example `CString::new()`.)

---

### Comment 346 by glandium
_2018-05-07T22:20:55Z_

You should rather use
```
#[repr(align(16))]
struct OverAligned16<T>(T);
```
and then `Vec<OverAligned16<T>>`.

---

### Comment 347 by gnzlbg
_2018-05-07T22:32:33Z_

> You should rather use

That depends. Suppose you want to use AVX intrinsics (256 bit wide, 32-byte alignment requirement) on a vector of `f32`s:

* `Vec<T, OverAlignedAlloc<U32>>` solves the problem, one can use AVX intrinsics directly on the vector elements (in particular, aligned memory loads), and the vector still derefs into a `&[f32]` slice making it ergonomic to use. 
* `Vec<OverAligned32<f32>>` does not really solve the problem. Each `f32` takes 32 bytes of space due to the alignment requirement. The padding introduced prevents the direct use of AVX operations since the `f32`s are not on continuous memory any more. And I personally find the deref to `&[OverAligned32<f32>]` a bit tedious to deal with.

For a single element in a `Box`, `Box<T, OverAligned<U32>>` vs `Box<OverAligned32<T>>`, both approaches are more equivalent, and the second approach might indeed be preferable. In any case is nice to have both options.

---

### Comment 348 by glandium
_2018-05-10T10:08:20Z_

Posted this wrt changes to the Alloc trait: https://internals.rust-lang.org/t/pre-rfc-changing-the-alloc-trait/7487

---

### Comment 349 by Amanieu
_2018-05-29T11:12:18Z_

The tracking post at the top of this issue is horribly out of date (was last edited in 2016). We need an updated list of active concerns to continue the discussion productively.

---

### Comment 350 by gnzlbg
_2018-05-29T11:34:26Z_

The discussion would also significantly benefit from an up-to-date design document, containing the current unresolved questions, and the rationale for the design decisions. 

There are multiple threads of diffs from "what's currently implemented on nightly" to "what was proposed in the original Alloc RFC" spawning thousands of comments on different channels (rfc repo, rust-lang tracking issue, global alloc RFC, internal posts, many huge PRs, etc.), and what's being stabilized in the `GlobalAlloc` RFC does not look that much from what was proposed in the original RFC.

This is something that we need anyways to finish updating the docs and the reference, and would be helpful in the current discussions as well.

---

### Comment 351 by Amanieu
_2018-05-29T20:15:03Z_

I think that before we even think about stabilizing the `Alloc` trait, we should first try implementing allocator support in all of the standard library collections. This should give us some experience with how this trait will be used in practice.

---

### Comment 352 by joshlf
_2018-05-29T20:21:23Z_

> I think that before we even think about stabilizing the `Alloc` trait, we should first try implementing allocator support in all of the standard library collections. This should give us some experience with how this trait will be used in practice.

Yes, absolutely. Especially `Box`, since we don't yet know how to avoid having `Box<T, A>` take up two words.

---

### Comment 353 by Amanieu
_2018-05-29T20:37:49Z_

> Yes, absolutely. Especially Box, since we don't yet know how to avoid having Box<T, A> take up two words.

I don't think we should worry the size of `Box<T, A>` for the initial implementation, but this is something that can be added later in a backward-compatible way by adding a `DeAlloc` trait which only supports deallocation.

Example:
```rust
trait DeAlloc {
    fn dealloc(&mut self, ptr: NonNull<Opaque>, layout: Layout);
}

trait Alloc {
    // In addition to the existing trait items
    type DeAlloc: DeAlloc = Self;
    fn into_dealloc(self) -> Self::DeAlloc {
        self
    }
}

impl<T: Alloc> DeAlloc for T {
    fn dealloc(&mut self, ptr: NonNull<Opaque>, layout: Layout) {
        Alloc::dealloc(self, ptr, layout);
    }
}
```

---

### Comment 354 by alexreg
_2018-05-29T20:39:12Z_

> I think that before we even think about stabilizing the Alloc trait, we should first try implementing allocator support in all of the standard library collections. This should give us some experience with how this trait will be used in practice.

I think @Ericson2314 has been working on this, per https://github.com/rust-lang/rust/issues/42774. Would be nice to get an update from him. 

---

### Comment 355 by joshlf
_2018-05-29T20:49:45Z_

> I don't think we should worry the size of `Box<T, A>` for the initial implementation, but this is something that can be added later in a backward-compatible way by adding a `DeAlloc` trait which only supports deallocation.

That's one approach, but it's not at all clear to me that it's definitely the best one. It has the distinct disadvantages, for example, that a) it only works when a pointer -> allocator lookup is possible (this isn't true of, e.g., most arena allocators) and, b) it adds significant overhead to `dealloc` (namely, to do the reverse lookup). It may end up being the case that the best solution to this problem is a more general-purpose effect or context system like [this proposal](https://internals.rust-lang.org/t/start-of-an-effects-system-rfc-for-async-etc-is-there-any-interest-in-this/7215) or [this proposal](https://internals.rust-lang.org/t/pre-pre-rfc-execution-context/7603). Or perhaps something different altogether. So I don't think we should assume that this will be easy to solve in a manner which is backwards-compatible with the current incarnation of the `Alloc` trait.

---

### Comment 356 by Amanieu
_2018-05-29T22:07:53Z_

@joshlf Considering the fact that `Box<T, A>` only has access to itself when it is dropped, this is the best thing that we can do with safe code only. Such a pattern might be useful for arena-like allocators which have a no-op `dealloc` and just free memory when the allocator is dropped.

For more complicated systems where the allocator is owned by a container (e.g. `LinkedList`) and managed multiple allocations, I expect that `Box` will not be used internally. Instead, the `LinkedList` internals will use raw pointers which are allocated and freed with the `Alloc` instance that is contained in the `LinkedList` object. This will avoid doubling the size of every pointer.

---

### Comment 357 by joshlf
_2018-05-29T22:19:11Z_

> Considering the fact that `Box<T, A>` only has access to itself when it is dropped, this is the best thing that we can do with safe code only. Such a pattern might be useful for arena-like allocators which have a no-op `dealloc` and just free memory when the allocator is dropped.

Right, but `Box` doesn't know that `dealloc` is no-op.

> For more complicated systems where the allocator is owned by a container (e.g. `LinkedList`) and managed multiple allocations, I expect that Box will not be used internally. Instead, the `LinkedList` internals will use raw pointers which are allocated and freed with the `Alloc` instance that is contained in the `LinkedList` object. This will avoid doubling the size of every pointer.

I think it would really be a shame to require people to use unsafe code in order to write any collections at all. If the goal is to make all collections (presumably including those outside of the standard library) optionally parametric on an allocator, and `Box` isn't allocator-parametric, then a collections author must either not use `Box` at all or use unsafe code (and keep in mind that remembering to always free things is one of the most common types of memory unsafety in C and C++, so it's difficult-to-get-right unsafe code at that). That seems like an unfortunate bargain.

---

### Comment 358 by eupp
_2018-06-12T19:43:24Z_

> Right, but Box doesn't know that dealloc is no-op.

Why wouldn't adapt what C++ `unique_ptr` does ? 
That is: to store pointer to allocator if it's "stateful", and do not store it if the allocator is "stateless"
(e.g. global wrapper around `malloc` or `mmap`).
This would require to split current `Alloc` traint into two traits: `StatefulAlloc` and `StatelessAlloc`. 
I realize that it is a very rude and inelegant (and probably someone has already proposed it in previous discussions).
Despite its inelegance this solution is simple and backward compatible (without performance penalties).

> I think it would really be a shame to require people to use unsafe code in order to write any collections at all. If the goal is to make all collections (presumably including those outside of the standard library) optionally parametric on an allocator, and Box isn't allocator-parametric, then a collections author must either not use Box at all or use unsafe code (and keep in mind that remembering to always free things is one of the most common types of memory unsafety in C and C++, so it's difficult-to-get-right unsafe code at that). That seems like an unfortunate bargain.

I'm afraid that an implementation of effect or context system which could allow one to write node-based containers like lists, trees, etc in safe manner might take too much time (if it's possible in principle).
I didn't see any papers or academic languages that tackle this problem (please, correct me if such works actually exist). 

So resorting to `unsafe` in implementation of node-based containers might be a necessary evil, at least in short-term perspective.
 

---

### Comment 359 by cramertj
_2018-06-12T19:56:44Z_

@eucpp Note that `unique_ptr` doesn't store an allocator-- [it stores a `Deleter`](http://en.cppreference.com/w/cpp/memory/unique_ptr):

> Deleter must be FunctionObject or lvalue reference to a FunctionObject or lvalue reference to function, callable with an argument of type unique_ptr<T, Deleter>::pointer`

I see this as roughly equivalent to us providing split `Alloc` and `Dealloc` traits.

---

### Comment 360 by eupp
_2018-06-12T20:07:49Z_

@cramertj Yes, you are right. Still, two traits are required - stateful and stateless `Dealloc`.

---

### Comment 361 by remexre
_2018-06-12T20:09:50Z_

Wouldn't a ZST Dealloc be sufficient?

On Tue, Jun 12, 2018 at 3:08 PM Evgeniy Moiseenko <notifications@github.com>
wrote:

> @cramertj <https://github.com/cramertj> Yes, you are right. Still, two
> traits are required - stateful and stateless Dealloc.
>
> —
> You are receiving this because you were mentioned.
> Reply to this email directly, view it on GitHub
> <https://github.com/rust-lang/rust/issues/32838#issuecomment-396716689>,
> or mute the thread
> <https://github.com/notifications/unsubscribe-auth/AEAJtWkpF0ofVc18NwbfV45G4QY6SCFBks5t8B_AgaJpZM4IDYUN>
> .
>


---

### Comment 362 by eupp
_2018-06-12T20:39:11Z_

> Wouldn't a ZST Dealloc be sufficient?

@remexre I suppose it would :)

I didn't know that rust compiler supports ZST out of box. 
In C++ it would require at least some tricks around empty base optimisation.
I'm pretty new at Rust so sorry for some obvious mistakes. 

---

### Comment 363 by SimonSapin
_2018-06-12T20:59:38Z_

I don’t think we need separate traits for stateful v.s. stateless.

With `Box` augmented with an `A` type parameter, it would contain a value of `A` directly, not a reference or pointer to `A`. That type can be zero-size fo a stateless (de)allocator. Or `A` itself can be something like a reference or handle to a stateful allocator that can be shared between multiple allocated objects. So instead of `impl Alloc for MyAllocator`, you might want to do something like `impl<'r> Alloc for &'r MyAllocator`

---

### Comment 364 by SimonSapin
_2018-06-12T21:03:07Z_

By the way, a `Box` that only knows how to deallocate and not how to allocate would not implement `Clone`.

---

### Comment 365 by cramertj
_2018-06-12T21:13:02Z_

@SimonSapin I'd expect that `Clone`ing would require specifying an allocator again, the same way that creating a new `Box` would (that is, it wouldn't be done using the `Clone` trait).

---

### Comment 366 by eupp
_2018-06-13T07:21:12Z_

@cramertj Wouldn't it be inconsistent compared to `Vec` and other containers that implement `Clone` ?
What are the downsides of storing instance of `Alloc` inside `Box` rather than `Dealloc` ?
Then `Box` might implement `Clone` as well as `clone_with_alloc`.

---

### Comment 367 by sfackler
_2018-06-13T16:00:20Z_

I don't this the split traits really affect Clone in a huge way - the impl would just look like `impl<T, A> Clone for Box<T, A> where A: Alloc + Dealloc + Clone { ... }`.

---

### Comment 368 by cramertj
_2018-06-13T16:46:48Z_

@sfackler I wouldn't be opposed to that impl, but I would also expect to have a `clone_into` or something that uses a provided allocator.

---

### Comment 369 by the8472
_2018-07-04T21:07:59Z_

Would it make sense to a `alloc_copy` method to `Alloc`? This could be used to provide faster memcpy (`Copy/Clone`) implementations for large allocations, e.g. by doing copy-on-write clones of pages.

---

### Comment 370 by joshlf
_2018-07-04T21:51:41Z_

That would be pretty cool, and trivial to provide a default implementation for.

---

### Comment 371 by glandium
_2018-07-04T21:57:42Z_

What would be using such an `alloc_copy` function? `impl Clone for Box<T, A>`?

---

### Comment 372 by the8472
_2018-07-04T22:03:24Z_

Yeah, ditto for `Vec`.

---

### Comment 373 by the8472
_2018-07-05T18:34:27Z_

Having looked into it some more it seems like approaches to create copy-on-write pages within the same process range between hacky and impossible, at least if you want to do it more than one level deep. So `alloc_copy` wouldn't be a huge benefit.

Instead a more general escape hatch that allows future virtual memory shenanigans might be of some use. I.e. if an allocation is large, backed by mmap anyway and stateless then the allocator could promise to be oblivious about future changes to the allocation. The user could then move that memory to a pipe, unmap it or similar things.
Alternatively there could be a dumb mmap-all-the-things allocator and a try-transfer function.

---

### Comment 374 by gnzlbg
_2018-07-06T12:01:33Z_

> Instead a more general escape hatch that allows future virtual memory 

Memory allocators (malloc, jemalloc, ...) do not generally let you steal any kind of memory from them,  and they do not generally let you query or change what the properties of the memory they own. So what does this general escape hatch has to do with memory allocators?

Also, virtual memory support differs greatly between platforms, so much that using virtual memory effectively often requires different algorithms per platform often with completely different guarantees. I have seen some portable abstractions over virtual memory, but I haven't seen one yet that wasn't crippled to the point of being useless in some situations due to their "portability". 

---

### Comment 375 by the8472
_2018-07-06T18:58:20Z_

You're right. Any such use-case (I was mostly thinking of platform-specific optimizations) is probably best served by using a custom allocator in the first place.

---

### Comment 376 by shanemikel
_2018-09-06T00:53:14Z_

Any thoughts on the Composable Allocator API described by Andrei Alexandrescu in his CppCon presentation?  The video is available on YouTube here: https://www.youtube.com/watch?v=LIb3L4vKZ7U (he starts to describe his proposed design around 26:00, but the talk is entertaining enough you may prefer to watch it through).

It sounds like the inevitable conclusion of all this is that collections libraries should be generic WRT allocation, and the app programmer himself should be able to compose allocators and collections freely at the construction site.

---

### Comment 377 by gnzlbg
_2018-09-06T09:27:24Z_

> Any thoughts on the Composable Allocator API described by Andrei Alexandrescu in his CppCon presentation?

The current `Alloc` API allows writing composable allocators (e.g. `MyAlloc<Other: Alloc>`) and you can use traits and specialization to achieve pretty much everything that's achieved in Andreis talk. However, beyond the "idea" that one should be able to do that, pretty much nothing from Andrei's talk can apply to Rust since the way Andrei builds the API sits on unconstrained generics + SFINAE/static if from the very start and Rust's generics system is completely different to that one.  

---

### Comment 378 by Amanieu
_2018-10-25T17:33:39Z_

I would like to propose stabilizing the rest of the `Layout` methods. These are already useful with the current global allocator API.

---

### Comment 379 by gnzlbg
_2018-10-25T17:44:59Z_

Are these all the methods that you mean?

* `pub fn align_to(&self, align: usize) -> Layout`
* `pub fn padding_needed_for(&self, align: usize) -> usize`
* `pub fn repeat(&self, n: usize) -> Result<(Layout, usize), LayoutErr>`
* `pub fn extend(&self, next: Layout) -> Result<(Layout, usize), LayoutErr>`
* `pub fn repeat_packed(&self, n: usize) -> Result<Layout, LayoutErr>`
* `pub fn extend_packed(&self, next: Layout) -> Result<(Layout, usize), LayoutErr>`
* `pub fn array<T>(n: usize) -> Result<Layout, LayoutErr>`

---

### Comment 380 by Amanieu
_2018-10-25T17:46:29Z_

@gnzlbg Yes.

---

### Comment 381 by SimonSapin
_2018-10-25T22:06:52Z_

@Amanieu Sounds ok to me, but this issue is already huge. Consider filing a separate issue (or even a stabilization PR) that we could FCP separately?

---

### Comment 382 by TimDiekmann
_2019-02-24T21:15:40Z_

from [Allocators and lifetimes](https://github.com/rust-lang/rfcs/blob/master/text/1398-kinds-of-allocators.md#allocators-and-lifetimes):

> 4. (for allocator impls): moving an allocator value must not invalidate its outstanding memory blocks.
> 
>     All clients can assume this in their code.
> 
>     So if a client allocates a block from an allocator (call it a1) and then a1 moves to a new place (e.g. vialet a2 = a1;), then it remains sound for the client to deallocate that block via a2.

Does this imply, that an Allocator *must* be `Unpin`?

---

### Comment 383 by SimonSapin
_2019-02-24T23:20:27Z_

Good catch!

Since the `Alloc` trait is still unstable, I think we still get to change the rules if we want to and modify this part of the RFC. But it is indeed something to keep in mind.

---

### Comment 384 by shanemikel
_2019-02-25T00:09:07Z_

@gnzlbg Yes, I'm aware of the huge differences in the generics systems, and that not everything he details is implementable in the same way in Rust.  I've been working on the library on-and-off since posting, though, and I'm making good progress.

---

### Comment 385 by withoutboats
_2019-02-25T02:12:22Z_

> Does this imply, that an Allocator _must_ be `Unpin`?

It doesn't.  `Unpin` is about the behavior of a type when wrapped in a `Pin`, there's no particular connection to this API.

---

### Comment 386 by TimDiekmann
_2019-02-25T09:05:07Z_

But can't `Unpin` be used to enforce the mentioned constraint?


---

### Comment 387 by TimDiekmann
_2019-02-25T13:24:39Z_

Another question regarding `dealloc_array`: Why does the function return `Result`? In the current implementation, this may fail in two cases:
- `n` is zero
- capacity overflow for `n * size_of::<T>()`

For the first we have two cases (as in the documentation, the implementer can choose between these): 
- The allocation returns `Ok` on zeroed `n` => `dealloc_array` should also return `Ok`. 
- The allocation returns `Err` on zeroed `n` => there is no pointer that can be passed to `dealloc_array`. 

The second is ensured by the following safety constraint:
> the layout of `[T; n]` must *fit* that block of memory.

This means, that we *must* call `dealloc_array` with the same `n` as in the allocation. If an array with `n` elements could be allocated, `n` is valid for `T`. Otherwise, the allocation would have failed.

**Edit:** Regarding the last point: Even if `usable_size` returns a higher value than `n * size_of::<T>()`, this is still valid. Otherwise the implementation violates this trait constraint:
> The block's size must fall in the range `[use_min, use_max]`, where:
> * [...]
> * `use_max` is the capacity that was (or would have been) returned when (if) the block was allocated via a call to `alloc_excess` or `realloc_excess`.

This only holds, as the trait requires an `unsafe impl`.


---

### Comment 388 by gnzlbg
_2019-02-25T16:56:49Z_

> For the first we have two cases (as in the documentation, the implementer can choose between these):
>
> * The allocation returns `Ok` on zeroed `n`

Where did you get this information from?

All `Alloc::alloc_` methods in the docs specify that the behavior of zero-sized allocations is undefined under their "Safety" clause. 

---

### Comment 389 by TimDiekmann
_2019-02-25T17:07:38Z_

[Docs of `core::alloc::Alloc`](https://doc.rust-lang.org/core/alloc/trait.Alloc.html) (highlighted relevant parts):

> A note regarding zero-sized types and zero-sized layouts: many methods in the `Alloc` trait state that allocation requests must be non-zero size, or else undefined behavior can result.
> 
> * However, some higher-level allocation methods **(`alloc_one`, `alloc_array`) are well-defined on zero-sized types and can optionally support them**: it is left up to the implementor whether to **return `Err`, or to return `Ok`** with some pointer.
> * If an `Alloc` implementation chooses to return `Ok` in this case (i.e. the pointer denotes a zero-sized inaccessible block) then that returned pointer must be considered "currently allocated". **On such an allocator, *all* methods that take currently-allocated pointers as inputs must accept these zero-sized pointers, *without* causing undefined behavior.**
> 
> * In other words, **if a zero-sized pointer can flow out of an allocator, then that allocator must likewise accept that pointer flowing back into its deallocation and reallocation methods**.

---

### Comment 390 by gnzlbg
_2019-02-25T17:15:39Z_

So one of the error conditions of `dealloc_array` is definitely suspicious:

```rust
/// # Safety
///
/// * the layout of `[T; n]` must *fit* that block of memory.
///
/// # Errors
///
/// Returning `Err` indicates that either `[T; n]` or the given
/// memory block does not meet allocator's size or alignment
/// constraints.
```

If `[T; N]` does not meet the allocator size or alignment constraints, then AFAICT it does not fit the block of memory of the allocation, and the behavior is undefined (per the safety clause). 

The other error condition is "Always returns `Err` on arithmetic overflow." which is pretty generic. It's hard to tell whether it is an useful error condition. For each `Alloc` trait implementation one might be able to come up with a different one that could do some arithmetic that could in theory wrap, so 🤷‍♂️ 

---

> [Docs of `core::alloc::Alloc`](https://doc.rust-lang.org/core/alloc/trait.Alloc.html) (highlighted relevant parts):

Indeed. I find it weird that so many methods (e.g. `Alloc::alloc`) state that zero-sized allocations are undefined behavior, but then we just provide `Alloc::alloc_array(0)` with implementation-defined behavior. In some sense `Alloc::alloc_array(0)` is a litmus test to check whether an allocator supports zero-sized allocations or not. 

---

### Comment 391 by TimDiekmann
_2019-02-25T17:28:56Z_

> If `[T; N]` does not meet the allocator size or alignment constraints, then AFAICT it does not fit the block of memory of the allocation, and the behavior is undefined (per the safety clause).

Yup, I think this error condition can be dropped as it's redundant. Either we need the safety clause or an error condition, but not both.

> The other error condition is "Always returns `Err` on arithmetic overflow." which is pretty generic. It's hard to tell whether it is an useful error condition.

IMO, it is guarded by the same safety clause as above; if the capacity of `[T; N]` would overflow, it does not *fit* that memory block to deallocate. Maybe @pnkfelix could elaborate on this?

> In some sense `Alloc::alloc_array(1)` is a litmus test to check whether an allocator supports zero-sized allocations or not.

Did you mean `Alloc::alloc_array(0)`?

---

### Comment 392 by gnzlbg
_2019-02-25T18:10:01Z_

> IMO, it is guarded by the same safety clause as above; if the capacity of `[T; N]` would overflow, it does not _fit_ that memory block to deallocate.

Note that this trait can be implemented by users for their own custom allocators, and that these users can override the default implementations of these methods. So when considering whether this should return `Err` for arithmetic overflow or not, one should not only focus on what the current default implementation of the default method does, but also consider what it might make sense for users implementing these for other allocators. 

> Did you mean `Alloc::alloc_array(0)`?

Yes, sorry.

---

### Comment 393 by TimDiekmann
_2019-02-25T18:36:58Z_

> Note that this trait can be implemented by users for their own custom allocators, and that these users can override the default implementations of these methods. So when considering whether this should return `Err` for arithmetic overflow or not, one should not only focus on what the current default implementation of the default method does, but also consider what it might make sense for users implementing these for other allocators.

I see, but implementing `Alloc` requires an `unsafe impl` and implementors has to follow the safety rules mentioned in https://github.com/rust-lang/rust/issues/32838#issuecomment-467093527.


---

### Comment 394 by SimonSapin
_2019-03-10T19:40:42Z_

Every API left pointing here for a tracking issue is the `Alloc` trait or related to the `Alloc` trait. @rust-lang/libs, do you feel it’s useful to keep this open in addition to https://github.com/rust-lang/rust/issues/42774?

---

### Comment 395 by joshlf
_2019-03-11T07:15:10Z_

Simple background question: What is the motivation behind the flexibility with ZSTs? It seems to me that, given that we know at compile-time that a type is a ZST, we can completely optimize out both the allocation (to return a constant value) and the deallocation. Given that, it seems to me that we should say one of the following:
- It's always up to the implementor to support ZSTs, and they can't return `Err` for ZSTs
- It's always UB to allocate ZSTs, and it's the caller's responsibility to short-circuit in this case
- There's some sort of `alloc_inner` method that callers implement, and an `alloc` method with a default implementation which does the short circuiting; `alloc` must support ZSTs, but `alloc_inner` may NOT be called for a ZST (this is just so that we can add the short-circuiting logic in a single place - in the trait definition - in order to save implementors some boilerplate)

Is there a reason that the flexibility we have with the current API is needed?

---

### Comment 396 by gnzlbg
_2019-03-11T08:04:22Z_

> Is there a reason that the flexibility we have with the current API is needed?

It's a trade-off.  Arguably, the Alloc trait is used more often than it is implemented, so it might make sense to make using Alloc as easy as possible by providing built-in support for ZSTs. 

This would mean that implementers of the Alloc trait will need to take care of this, but more importantly to me that those trying to evolve the Alloc trait will need to keep ZSTs in mind on every API change. It also complicates the docs of the API by explaining how ZSTs are (or could be if it is "implementation defined") handled. 

C++ allocators pursue this approach, where the allocator tries to solve many different problem. This did not only make them harder to implement and harder to evolve, but also harder for users to actually use because of how all these problems interact in the API. 

I think that handling ZSTs, and allocating/deallocating raw memory are two orthogonal and different problems, and therefore we should keep the Alloc trait API simple by just not handling them.

Users of Alloc like libstd will need to handle ZSTs, e.g., on each collection. That's definitely a problem worth solving, but I don't think the Alloc trait is the place for that. I'd expect an utility to solve this problem to pop out within libstd out of necessity, and when that happens, we can maybe try to RFC such an utility and expose it in std::heap. 

---

### Comment 397 by joshlf
_2019-03-11T15:44:08Z_

That all sounds reasonable.

> I think that handling ZSTs, and allocating/deallocating raw memory are two orthogonal and different problems, and therefore we should keep the Alloc trait API simple by just not handling them.

Doesn't that imply that we should have the API explicitly not handle ZSTs rather than be implementation-defined? IMO, an "unsupported" error is not very helpful at runtime since the vast majority of callers will not be able to define a fallback path, and will therefore have to assume that ZSTs are unsupported anyway. Seems cleaner to just simplify the API and declare that they're _never_ supported.

---

### Comment 398 by burdges
_2019-03-11T16:14:53Z_

Would specialization be used by `alloc` users to handle ZST?  Or just `if size_of::<T>() == 0` checks?

---

### Comment 399 by joshlf
_2019-03-11T16:17:10Z_

> Would specialization be used by `alloc` users to handle ZST? Or just `if size_of::<T>() == 0` checks?

The latter should be sufficient; the appropriate code paths would be trivially removed at compile time.

---

### Comment 400 by gnzlbg
_2019-03-11T16:17:24Z_

> Doesn't that imply that we should have the API explicitly not handle ZSTs rather than be implementation-defined?

For me, an important constraint is that if we ban zero-sized allocations, the `Alloc` methods should be able to assume that the `Layout` passed to them is not zero-sized. 

There are multiple ways to achieve this. One would be to add another `Safety` clause to all `Alloc` methods stating that if the `Layout` is zero-sized the behavior is undefined. 

Alternatively, we could ban zero-sized `Layout`s, then `Alloc` doesn't need to say anything about zero-sized allocations since these cannot safely happen, but doing that would have some downsides. 

For example, , some types like `HashMap` build up the `Layout` from multiple `Layout`s, and while the final `Layout` might not be zero-sized, the intermediate ones might be (e.g. in `HashSet`). So these types would need to use "something else" (e.g. a `LayoutBuilder` type) to build up their final `Layout`s, and pay for a "non-zero-sized" check (or use an `_unchecked`) method when converting to `Layout`. 

> Would specialization be used by alloc users to handle ZST? Or just if size_of::<T>() == 0 checks?

We can't specialize on ZSTs yet. Right now all code uses `size_of::<T>() == 0 `.

---

### Comment 401 by joshlf
_2019-03-11T16:21:26Z_

> There are multiple ways to achieve this. One would be to add another `Safety` clause to all `Alloc` methods stating that if the `Layout` is zero-sized the behavior is undefined.

It'd be interesting to consider whether there are ways to make this a compile-time guarantee, but even a `debug_assert` that the layout is non-zero-sized should be sufficient to catch 99% of bugs.

---

### Comment 402 by brson
_2019-04-04T21:40:44Z_

I haven't paid any attention to discussions about allocators, so sorry about that. But I've long wished that the allocator had access to the type of the value its allocating. There may be allocator designs that could use it.

---

### Comment 403 by TimDiekmann
_2019-04-04T21:54:49Z_

Then we'd probably have the same issues as C++ and it's allocator api. 

---

### Comment 404 by gnzlbg
_2019-04-05T14:07:41Z_

> But I've long wished that the allocator had access to the type of the value its allocating. T

What do you need this for?

---

### Comment 405 by raphaelcohn
_2019-04-12T07:14:34Z_

@gnzblg @brson Today I had a potential use case for a knowing _something_ about the type of value being allocated.

I'm working on a global allocator which can be switched between three underlying allocators - a thread local one, a global one with locks, and one for coroutines to use - the idea being I can constrain a coroutine representing a network connection to a maximum amount of dynamic memory usage (in the absence of being able to control allocators in collections, esp in third party code)\*.

It'd be possibly be handy to know if I'm allocating a value that might move across threads (eg Arc) vs one that won't. Or it might not. But it is a possible scenario. At the moment the global allocator has a switch which the user uses to tell it from which allocator to make allocations (not needed for realloc or free; we can just look at the memory address for that).

\*[It also allows me to use NUMA local memory wherever possible without any locking, and, with a 1 core 1 thread model, to cap total memory usage].

---

### Comment 406 by gnzlbg
_2019-04-12T08:47:24Z_

@raphaelcohn 

> I'm working on a global allocator

I don't think any of this would (or could) apply to the `GlobalAlloc` trait, and the `Alloc` trait already has generic methods that can make use of type information (e.g. `alloc_array<T>(1)` allocates a single `T`, where `T` is the actual type, so the allocator can take the type into account while performing the allocation). I think it would be more useful for the purposes of this discussion to actually see code implementing allocators that make use of type information. I haven't heard any good argument about why these methods need to be part of some generic allocator trait, as opposed to just being part of the allocator API, or some other allocator trait.

---

### Comment 407 by gnzlbg
_2019-04-12T09:36:51Z_

I think it would also be very interesting to know which of the types parametrized by `Alloc` do you intend to combine with allocators that use type information, and what do you expect to be the outcome.

AFAICT, the only interesting type for that would be `Box` because it allocates a `T` directly.  Pretty much all other types in `std` never allocate a `T`, but some private internal type that your allocator can't know anything about. For example,  `Rc` and `Arc` could allocate `(InternalRefCounts, T)`, `List` / `BTreeSet` / etc. allocate internal node types, `Vec`/`Deque`/... allocate arrays of `T`s, but not `T`s themselves, etc.

For `Box` and `Vec` we could add in backwards compatible ways a `BoxAlloc` and an `ArrayAlloc` trait with blanket impls for `Alloc` that allocators could specialize to hijack how those behave, if there is ever a need to attack these problems in a generic way. But is there a reason why providing your own `MyAllocBox` and `MyAllocVec` types that conspire with your allocator to exploit type information isn't  a viable solution ?

---

### Comment 408 by TimDiekmann
_2019-05-04T12:40:01Z_

As we now have a [dedicated repository for the allocators WG](https://github.com/rust-lang/wg-allocators), and the list in the OP is out of date, this issue may be closed to keep discussions and tracking of this feature in one place?

---

### Comment 409 by alexcrichton
_2019-05-06T15:16:48Z_

A good point @TimDiekmann! I'm going to go ahead and close this in favor of discussion threads in that repository.

---

### Comment 410 by SimonSapin
_2019-05-06T15:50:46Z_

This is still the tracking issue that some `#[unstable]` attribute point to. I think it should not be closed until these features have been either stabilized or deprecated. (Or we could change the attributes to point to a different issue.)

---

### Comment 411 by jethrogb
_2019-05-06T17:30:56Z_

Yeah unstable features referenced in git master should definitely have an open tracking issue.

---

### Comment 412 by Gankra
_2019-05-06T17:47:57Z_

Agreed. Also added a notice and link to the OP.

---

### Comment 413 by adsnaider
_2021-11-23T03:27:56Z_

Figured I would put this in here. I ran into a compiler bug when using the allocator_api feature with Box: #90911 

---

### Comment 414 by RalfJung
_2022-07-04T03:30:17Z_

> Figured I would put this in here. I ran into a compiler bug when using the allocator_api feature with Box: https://github.com/rust-lang/rust/issues/90911

In fact this feature is littered with ICEs. That's because it fits quite badly with MIR and the MIR-consuming parts of the compiler -- codegen, Miri all need to do lots of special-casing for `Box`, and adding an allocator field made it all a lot worse. See https://github.com/rust-lang/rust/issues/95453 for more details. IMO we shouldn't stabilize `Box<T, A>` until these design issues are resolved.

---

### Comment 415 by scottmcm
_2022-09-15T05:32:58Z_

`Box` being [`#[fundamental]`](https://github.com/rust-lang/rust/issues/29635#issuecomment-1247598385) means that one [can](https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=a5107d38cef18e25be4bd7ef46ad522d) implement core traits for `Box<i32, MyCustomAllocator>`, and that probably ought to be prevented before stabilizing the type parameter.

---

### Comment 416 by thomcc
_2022-09-15T05:43:48Z_

I have a lot of concerns about the current API -- t's not a good fit for existing allocators leading to perf losses, and it's story is quite poor when used with dynamic dispatch (a common use case for allocators).

It's a bit surprising that this apparently has finished FCP with merge disposition, but IMO there's more than just the issue around `#[fundamental]` that needs work here.

---

### Comment 417 by RalfJung
_2022-09-15T05:58:43Z_

I'd also prefer if we had at least a solid plan for how to properly solve the problems arising from Box being both a primitive type (for purposes like noalias and align attributes) and having arbitrary user-defined data inside it. It looks like for now the ICE wack-a-mole slowed down but architecturally in the compiler it is still somewhat messy.

Things did get a lot better since my previous message thanks to the new "derefer" MIR pass, but there are a bunch of remaining rough edges.

---

### Comment 418 by CraftSpider
_2022-09-16T00:38:57Z_

I'd also chime in that there was, fairly recently, discussion on zulip about a storages proposal. Stabilizing the current API would likely render implementing such a proposal in a backwards-compatible way impossible. I personally would like to at least have time for such a proposal to be made and decided on before stabilizing allocators as-is

---

### Comment 419 by programmerjake
_2023-01-18T18:00:33Z_

the FCP labels should probably be removed since they appear to be out of date

---

### Comment 420 by 
_2023-03-24T00:21:43Z_

2016

---

### Comment 421 by Amanieu
_2023-05-19T16:47:09Z_

We discussed this in the libs meeting yesterday and would like to, as a first step, stabilize just the `Allocator` trait without stabilizing its use in standard library container just yet.

There are several blockers that need to be resolved before this can happen:
- Issues around the exact lifetime of allocations for allocators with lifetimes needs to be resolved. Specifically:
  - #94069
  - #90822
- Add a method to the `Allocator` trait to determine whether 2 allocator instances are equivalent:
  - https://github.com/rust-lang/wg-allocators/issues/109
- Deprecate `GlobalAlloc` and change `#[global_allocator]` to require `Allocator` instead.
  - https://github.com/rust-lang/wg-allocators/issues/43
  - https://github.com/rust-lang/wg-allocators/issues/21

We are specifically choosing to defer the following features:
- [Adding an `Allocator` generic parameter to existing collections](https://github.com/rust-lang/wg-allocators/issues/7): There are still complex issues that need to be resolved here, such as it not being implemented on some type, or https://github.com/rust-lang/wg-allocators/issues/90.
- [Storage proposal](https://internals.rust-lang.org/t/pre-rfc-storage-api/18822): This can be added later in a backwards-compatible way since all allocators implement `Storage`.
- [Split deallocator trait](https://github.com/rust-lang/wg-allocators/issues/112): This can be added later in a backwards-compatible way since all allocators implement `Deallocator`.

---

### Comment 422 by Ericson2314
_2023-05-19T16:57:17Z_

Is the thinking that we don't think adding allocator parameters to collections will teach us anything about the allocator trait itself? Not necessarily agreeing or disagreeing, just want to be explicit about it.

---

### Comment 423 by Amanieu
_2023-05-19T21:34:18Z_

Sort of. We think the `Allocator` trait is mostly ready, but we would like to get a better idea of how it is used with collections in the ecosystems so that we can apply this when adding support to the types in the standard library.

---

### Comment 424 by Jules-Bertholet
_2023-05-19T21:55:07Z_

There is also the issue that you don't want to change the stdlib collections in a way that would make it hard to support `Storage` later.

---

### Comment 425 by thomcc
_2023-05-19T21:59:48Z_

Is the allocator trait really ready? I've tried to use it a few times in the past and it felt really awkward and painful (I've shared a writeup before, earlier in this thread, focused around the pain of trying to use `dyn Allocator`: https://hackmd.io/ld8wv8kHQaKF_YsU6r8CKQ).

It also doesn't map well to existing system allocator APIs, or to our existing GlobalAlloc trait. And relevant to even just the standard library usage in particular, the stuff around excess is almost unusable due to not being able to distinguish between cases that desire excess capacity and cases that don't.

I get the desire to push for stabilization on a long-unstable feature that many users want, but I don't know that this is actually a trait in a position to be stabilized without us regretting it later.

---

### Comment 426 by Amanieu
_2023-05-19T22:04:49Z_

> Is the allocator trait really ready? I've tried to use it a few times in the past and it felt really awkward and painful (I've shared a writeup before, earlier in this thread, focused around the pain of trying to use `dyn Allocator`: [hackmd.io/ld8wv8kHQaKF_YsU6r8CKQ](https://hackmd.io/ld8wv8kHQaKF_YsU6r8CKQ)).

I did read that, but you don't propose any concrete way to improve upon what we currently have. It's also difficult for me to understand how well the solutions that you propose apply since I am not familiar with your code base.

---

### Comment 427 by thomcc
_2023-05-19T22:18:27Z_

That's true, I think my concrete suggestion for that case would be to follow more along the design of RawWaker, which manages to allow dynamic dispatch without requiring an allocator.

---

### Comment 428 by withoutboats
_2023-05-19T22:28:34Z_

(NOT A CONTRIBUTION)

> Is the allocator trait really ready? I've tried to use it a few times in the past and it felt really awkward and painful (I've shared a writeup before, earlier in this thread, focused around the pain of trying to use dyn Allocator: https://hackmd.io/ld8wv8kHQaKF_YsU6r8CKQ).

I don't see any way an API like waker would improve the situation. You wrote:

> … Especially because in the majority of cases, you shouldn’t even need to allocate anything for this, since it’s a ZST anyway. And for that matter, for these you shouldn’t need to maintain a refcount)…
>
> For completeness, there’s also Vec<T, &'static dyn Allocator>. This works, but is either dangerous or unflexible.

If all you care about are ZST allocators that live the length of the program and you want dynamic dispatch, `&'static dyn Allocator` is absolutely the right type for you. They're ZSTs, you can put one in a static and use its allocator implementation.

If you want them to be able to hold data and live for less than the length of your program (like an arena), then you need `&'a dyn Allocator` and, yea, you need to manage that lifetime. Them's the breaks.

A waker-like API would force dynamic dispatch on everyone, and the only thing it would allow is using raw pointers as the receiver type.

---

Allocator should probably be implemented for all of the pointer types to a type that implements Allocator, not just shared reference.

---

I'm personally not very convinced `AllocError` pulls its weight and would weigh the option of just using `Option`. You know you're never going to add fields to `AllocError` because it would pessimize calls to malloc. What are the benefits of `AllocError`?

---

### Comment 429 by Lokathor
_2023-05-19T22:31:48Z_

FooLibError can impl From<AllocError> and then `?` will Just Work.

---

### Comment 430 by RalfJung
_2023-05-20T07:05:05Z_

> Allocator should probably be implemented for all of the pointer types to a type that implements Allocator, not just shared reference.

FWIW that shared reference impl is a perf footgun, not sure if we want to keep it: https://github.com/rust-lang/rust/issues/98232.

---

### Comment 431 by yanchith
_2023-06-09T10:51:56Z_

> If all you care about are ZST allocators that live the length of the program and you want dynamic dispatch, &'static dyn Allocator is absolutely the right type for you. They're ZSTs, you can put one in a static and use its allocator implementation.

> If you want them to be able to hold data and live for less than the length of your program (like an arena), then you need &'a dyn Allocator and, yea, you need to manage that lifetime. Them's the breaks.

@thomcc Not sure if helpful to you, but in my engine I have built an abstraction for "allocator + data allocated in it".

https://gist.github.com/yanchith/113bca60ccf86b79ad7e4b0ddec98c64

This lets me pretend that `'a` is `'static` and stops the lifetime contagion. Of course, this is unsafe , but the tradeoff I made is to make resetting the allocator unsafe so that everything else, even leaking, can be safe. The invariant for resetting is that the allocator must not have been used for any data that's not tracked by the abstraction.

This mostly works very well, but there's still a few annoyances, like the data inside the abstraction always having to be initialized and usable, so every in a couple of places I have to pass in a constructor function that creates the data from the allocator.

(The code itself is tailored to work with reference to one specific allocator, but I can imagine making it work with &dyn Allocator too)

---

### Comment 432 by thomcc
_2023-06-10T14:16:41Z_

So, I still think the support for `dyn` is not great but honestly it's been long enough since I wrote that that I don't care to dig up the details (they weren't static ZSTs, nor were they borrowed, they were reference counted but allocating memory for the `Rc` requires an allocator... Anyway, it's not important), and I think it can be solved later anyway.

More broadly it is by far not the only issue I hit. Here's one that's easy for me to argue for, that I expect nearly everybody to hit as they use the API for the first time (even now when writing this I messed up multiple times, despite thinking very intently about the issue). And the only benefit we get for it is a mild theoretical one:

### Supporting zero-sized allocations

`Allocator`, allows users to call it with zero-sized allocation requests, since this allows `allocate` to be a safe function. This is a change from `GlobalAlloc`, which forbids this, with the constraint that its input `Layout` must not be zero-sized.

This has been [discussed](https://github.com/rust-lang/wg-allocators/issues/16) in the [past](https://github.com/rust-lang/wg-allocators/issues/38). The discussions focused on performance[^perf], and suggested the concerns about difficult checking were solved by adding an extra method (which is insufficient for reasons I'll get into, and no longer exists anyway).

[^perf]: The performance measurments and arguments seem to have failed to account for dynamic dispatch, but perhaps `Allocator` was not object safe at that point. (It's also really hard to measure branch prediction overhead in a microbenchmark, due to branch predictors being more sophisticated the benchmarks most folks write). Either way this isn't a part of my argument, although I do think it matters.

There are two reasons I believe this to be a mistake:
1. Making `Allocator::allocate` safe is useless. There is no way to use the memory it returns without unsafe code, and realistic uses will want to `deallocate` that memory later, which requires calling an unsafe method.

2. The impact of this decision is that every single method on `Allocator` must now be certain that it is handling zero-sized layouts correctly. This ends up being really awkward -- suddenly when implementing `Allocator::shrink`, you must start by determining "is this actually `shrink`, or is this a round-about way of calling `deallocate`" (e.g. is it being shrunk to zero-sized layout), similarly "grow" needs to determine if it's actually being asked to do the initial allocation.

Concretely, a correct[^1] implementation of `Allocator` that overrides all methods (obviously if you use our defaults it will be fine, but they may be significantly slower than what is possible), is very likely[^2] to need to end up handling zero sized allocations in the following manner, or something close:

[^1]: I'm considering "returns error on all zero-sized allocation" to be a form
    of incorrectness, as none of the stdlib allocators behave this way. It's
    obviously sound to do so, however.

[^2]: The main case where you can get away without these checks is a bump allocator, which has no deallocation or resizing, and can be written to avoid the branch on alloc size==0, but often will have to trade a small amount of wasted space (to align the zero-sized allocation) in order to do so. Either way such an allocator is not harmed by changing this design (if anything, their design argues that we *should* care about adding branching into these functions). Besides, most allocator authors will not be calling bump allocators, so I don't think it should be considered representative.

- In `allocate` and `allocate_zeroed`, handle the case where the layout is
  zero-sized, and return the correct dangling pointer.
    ```rs
    if layout.size() == 0 {
        return Ok(NonNull::slice_from_raw_parts(layout.dangling(), 0));
    }
    ```
- In `deallocate`, handle the case where the layout is zero-sized, and do
  nothing:
    ```rs
    if layout.size() == 0 {
        return;
    }
    ```
- In `grow`, handle the case where `old` is zero-sized, and the case where both
  `old` and `new` are zero-sized (although the "both zero" case is handled
  inside `self.allocate` already):
    ```rs
    if old.size() == 0 {
        // Note: `self.allocate` handles `new.size() == 0`
        return self.allocate(new);
    }
    ```
- `grow_zeroed` is the same as `grow`, but with `allocate_zeroed` instead of
  `allocate`:
    ```rs
    if old.size() == 0 {
        // Note: `self.allocate` handles `new.size() == 0`
        return self.allocate_zeroed(new);
    }
    ```
- And finally, in `shrink`: `new` may be zero-sized, and if it is, you need to call `deallocate` and then return dangling pointer (based on `new.dangling()`, not `old` -- I got this wrong when writing this example):
    ```rs
    if new.size() == 0 {
        // Note: `self.deallocate` handles `old.size() == 0`
        self.deallocate(ptr, old);
        return Ok(NonNull::slice_from_raw_parts(new.dangling(), 0));
    }
    ```

*(A complete version is [here](https://gist.github.com/thomcc/4211309b4479fb3146d86f2fdc50a624))*

Individually, none of these seem that bad, but they're actually fairly tricky to get right, and took me a few tries. Online, I wasn't able to find folks who got it right either, although usually they failed to check at all.

I think there are cases where we'd say this is fine for a low-level unsafe trait, but we really get no benefit from it, and even have already made the decision once to have it be unsafe.

---

Anyway, I'll leave it there for now. Basically, I think we actually got it right with `GlobalAlloc::alloc` being unsafe -- `allocate` being safe is useless, and it's not worth complicating (nearly) every implementation this way.

*(I started a zulip thread for this <https://rust-lang.zulipchat.com/#narrow/stream/197181-t-libs.2Fwg-allocators/topic/Forbidding.20Zero-sized.20layouts.20in.20Allocator.3A.3Aallocate.2E>)*

---

### Comment 433 by thomcc
_2023-06-10T15:33:34Z_

I wrote up another that has bugged me with Vec for a while. Basically it's unfortunate that `grow()` is forced to copy the whole capacity of the vec rather than just the part in use.

Zulip: https://rust-lang.zulipchat.com/#narrow/stream/197181-t-libs.2Fwg-allocators/topic/Allocator.20growth.20method.20and.20length.20param

I mention this because while it's minor, it also seems like adding it after stabilization would be a lot worse[^1], and it can show up in performance profiles.

[^1]: Either we default the impl to calling the existing grow method (bad because we don't get to use it to fix Vec's problem), or we default it to allocate new+copy+dealloc old, which hurts most custom allocators far more than this problem.




---

### Comment 434 by thomcc
_2023-06-10T21:47:46Z_

Oh, the current zero-sized semantics are even more problematic than I realized. @Amanieu pointed out that you could handle zero-sized allocations in your `Allocator` by bumping them up to some non-zero value.

I had forgotten this but yeah it's intended to work -- allocators are supposed to be able to round/bump up their allocations, and so long as they do so consistently and don't require callers reflect that increase in the Layouts they use later on everything is supposed to be fine.

Unfortunately, this means zero-sized allocations are:
1. no longer free in terms time *or* space.
2. something you must deallocate.

The first would be terribly annoying for the end user, since now `Box::new_in` needs to loose its guarantees, but... the second is a bigger problem.

Concretely, it's terrible for container authors as well, since it means that if they decide not to handle zero sized types (and let it be handled entirely by the allocator), they can no longer use something like `capacity == 0` as the constraint to determine whether or not they need to free the memory.

Consider the following situation (which is a specific instance of a more general problem, rather than a bug that only is in Vec):
1. An allocator `BumpZeroAllocator` exists that implements the "bump up zero-sized allocation" strategy.
2. You have a `Vec<T, BumpZeroAllocator>`, which has a zero length but not a zero capacity.
3. You call `shrink_to_fit()` on the vec.
4. The vec requests an `allocator.shrink` call happen to reduce the size from whatever it was, into a zero-sized layout (because the Vec had len==0)
5. The allocator bumps the zero-sized target layout to be nonzero, but otherwise services the allocation exactly, returning a `NonZero<[u8]>` with a non-zero length (e.g. a nonzero excess).
6. The vec (well, its rawvec) ignores the NonZero excess, because it's allowed to do so. It stores zero as the capacity, and the result pointer as the pointer.
7. Later, when Vec is dropped, it checks if the capacity was zero to know if the allocation needs to be freed. It is zero, so it does not get freed.
8. Unfortunately, the allocation did need to be freed, and didn't get freed.

This is a real bug we have today: https://play.rust-lang.org/?version=nightly&mode=debug&edition=2021&gist=f47a46ddf21ef83a3146197af79574e5. Note that it's not specific to shrink, as it could happen with any zero-sized allocation request. Also very important to point out: This is not a nefarious Allocator impl, and it's not misusing or incorrectly implementating any part the API as far as I know -- this is intended to be a legal Allocator implementation.

That said, I don't see a good way to fix this without making all our collection types check for zero-sized allocations prior to calling their allocator. Or start carving out ways zero-sized layouts need to be handled.

---

TLDR: If the allocator is responsible for handling zero-sized layouts, it becomes very very hard to implement correct container types without handling zero-sized layouts on the container side of the API (in addition to what I said previously about the allocator-side needing to handle them).

Vec has a leak caused by this in from completely legal (one that was intended by design) allocator implementation, and to fix it we'd need to basically always call `Allocator` with non-zero layouts, meaning often both sides of the API would have the checks...

Additionally, allowing this would probably make us have to give up the comment on [`Box::new_in`](https://doc.rust-lang.org/nightly/std/boxed/struct.Box.html#method.new_in) that says "This doesn’t actually allocate if T is zero-sized.", because the underlying allocator might.

In general, the semantics are hard to work with for implementors of allocators (in a more obvious and annoying way), hard to work with for users of allocator (in a much more subtle way), and push both sides of the API to need to implement the same checks. And they also give us almost no practical benefit, since the API is still unsafe in practice. So, I get that the design has a ton of inertia, but... yeah.

---

### Comment 435 by Amanieu
_2023-06-11T00:17:51Z_

I think that both `Box` and `Vec` need to be fixed to check for a capacity of 0 and not allocate memory in this case. In particular, `RawVec::shrink` needs to check if it is shrinking to a capacity of 0 and free its backing memory instead of shrinking it.

In essence, it is the allocator caller's responsibility to properly handle ZSTs *before* calling into the allocator, if they wish to guarantee no allocation for ZSTs. Allocators are required to properly handle zero-sized layouts in that the returned allocation can later be deallocated, but they are not required to ensure that zero-sized allocations do not consume memory. In fact, allocators should avoid special-casing zero-sized layouts and let the caller deal with it.

This is not an uncommon stance. For example, most `malloc` implementations in libc (including glibc, dlmalloc, musl) will allocate memory for `malloc(0)` so that the resulting pointer is non-null and can be passed to `free`.

---

### Comment 436 by thomcc
_2023-06-11T01:25:14Z_

> In particular, RawVec::shrink needs to check if it is shrinking to a capacity of 0 and free its backing memory instead of shrinking it.

At the point where every call-site has to special case this, we should just admit that the part of the allocator interface that tries to make zero-sized layouts allowed is a bad part of that interface. But more broadly this seems extremely unfortunate to me.

The problem is that you're basically suggesting that it's fine if `Box`/`Vec` have a convention where they special case zero-sized layouts, and that this can be an implementation detail of their use of allocators, which other container authors can either decide to adopt, or not.

I don't know that that works, I think it's very surprising and confusing, but concretely: `Box` and `Vec` promise that you're able to allocate memory using the allocator separately and construct them from the memory it gives back. This promise currently applies only to `Global` but is quite nice, and would an unfortunate limitation on the allocator interface if it lost this. However, this means that everybody using `Allocator` pretty much has to do it like `Box`/`Vec` do it.

> In essence, it is the allocator caller's responsibility to properly handle ZSTs before calling into the allocator, if they wish to guarantee no allocation for ZSTs

Yeah so I don't get the point of this argument. What do we get out of keeping the API the way it is now? It's clearly error-prone to use (std got it wrong), I can't speak as concretely but I'm telling you that I've found it very error-prone to write allocators with the current API.

The status quo will end up with us checking before calling the allocator, and then the allocator checking after being called. This seems like clearly the worst of all possible situations.

---

I mean, The easy way to solve all of these problems IMO is just to forbid zero-sized layouts in the allocator interface, and document that the process for "allocating" a zero-sized block of memory is `layout.dangling()`. That fixes all of it, and makes it very clear to the users of the allocator trait what their responsibilities are.

Is it possible to have an allocator trait without that limitation? Yeah, it is, and it would look like the one we have. It's just very hard to use, easy to misuse, error-prone, and gives very little benefits.

If you're in a case where the allocator you really need to call some custom allocator and don't want to pay for a zero-size check on either side, I think it's fine to just have that call not go throught the Allocator interface.

---

### Comment 437 by nbdd0121
_2023-06-11T11:43:15Z_

We can still keep `allocate` function safe by adding `NonZeroLayout`.

---

### Comment 438 by thomcc
_2023-06-12T05:24:05Z_

I'm in favor of that. It makes what to do clear and gives a place to put the documentation about how to "allocate" a zero-sized layout.

---

### Comment 439 by RalfJung
_2023-06-16T13:28:26Z_

> Concretely, it's terrible for container authors as well, since it means that if they decide not to handle zero sized types (and let it be handled entirely by the allocator), they can no longer use something like capacity == 0 as the constraint to determine whether or not they need to free the memory.

That seems quite unsurprising? You have to either consistently handle size zero in your container or let it be handled by the allocator, mix-and-match cannot work.

Arguably RawVec is at fault here for not properly tracking if it actually has any memory that needs to be given back to the allocator. If zero-sized allocation was UB, presumably shrink-to-0 would also be UB, so the RawVec code would still be wrong, but for different reasons?

---

### Comment 440 by victoryaskevich
_2023-06-16T14:01:55Z_

Hey guys, I just created a new issue to discuss. https://github.com/rust-lang/wg-allocators/issues/114

I am not actually pushing towards splitting the trait, but I would like this issue to be addressed.

---

### Comment 441 by thomcc
_2023-06-23T06:37:35Z_

> If zero-sized allocation was UB, presumably shrink-to-0 would also be UB, so the RawVec code would still be wrong, but for different reasons?

Yes, we'd need to use NonZeroLayout (or whatever) there too. I think we'd need to use it everywhere?

---

### Comment 442 by RalfJung
_2023-06-25T07:11:19Z_

My question was: isn't the RawVec code wrong either way in how it handles shrink-to-0, both with the current Allocator trait and with your proposed "non zero" variant? You said the current trait is a footgun, but it seems your proposal doesn't actually resolve that footgun.

---

### Comment 443 by thomcc
_2023-06-25T07:18:16Z_

It solves it by forcing you to use deallocation rather than shrinking to a zero-sized allocation, because creating the zero-sized layout would fail (or be UB if done improperly).

---

### Comment 444 by RalfJung
_2023-06-25T13:18:13Z_

So that's the same solution as with an allocator that does support zero size.

Is the point that supporting zero size doesn't bring any *benefit* for types like Vec that aim to not invoke the allocator in `new`? You seemed to say that the current Allocator contract makes things worse for Vec (compared to a non-zero-size Allocator) and I don't see how.

---

### Comment 445 by thomcc
_2023-06-26T10:15:14Z_

Yes, but that's highly non-obvious[^1], and will be a subtle footgun that's hard to get right -- specifically, this is hard to get right for both the user of the Allocator API, and implementer of the Allocator API. In concrete terms, you will often end up with checks on both side of the API boundary, which is definitely a sign that something is wrong.

I didn't say that the API cannot be made to work in the way it currently is, just that it's a footgun on both ends.

[^1]: Your earlier comment that it's unsurprising is counter to my experience that almost everybody using this API has gotten it wrong, including the stdlib.

---

### Comment 446 by RalfJung
_2023-06-26T11:46:53Z_

IMO that the source of the problem here is `Vec` insisting on not interacting with the allocator in `Vec::new`. This means it *must* always know whether it has something to give back to the allocator or not. That's an important piece of state to track. If `Vec` used a capacity of `Option<usize>` where `None` indicates "not allocated" this would be trivial, but `Vec` chose to represent `None` as `0` and that's where things start to become subtle. The shrinking logic seems to currently get that wrong, but I wouldn't blame the allocator API for that -- it's `Vec` that chose to be "clever" in its state representation here.

In most cases it is probably perfectly fine to have a data structure invariant saying that the data pointer is always something that was returned by `alloc`, and hence can be freed by `dealloc`. Then grow/shrink can be implemented in the obvious way without any checks on the data structure side. That would be a `Vec` that just always calls `alloc` in `new` and `dealloc` in `drop`. However, the real `Vec` chooses to not follow that path and instead has an invariant saying "if the capacity is non-zero, then there is some memory to give back to the allocator, else there is not". `Vec` then has to live with the consequences of that choice, in particular it uses capacity 0 as a sentinel itself and hence has to always special-case this situation everywhere. Why would we punish every user of `Allocator` (having them special case capacity 0) for this somewhat special choice `Vec` made?

Or is this common enough that we want to tailor the entire allocator trait towards data structures that use capacity 0 as a sentinel themselves to avoid calling `alloc` in `new`?

---

### Comment 447 by the8472
_2023-06-26T14:41:50Z_

> Or is this common enough that we want to tailor the entire allocator trait towards data structures that use capacity 0 as a sentinel themselves to avoid calling alloc in new?

Most collections do this, yeah. To enable an empty collection to be about as cheap as an Option::None, to make it `const fn new()` and to make construction panic-free.

Though as @thomcc already mentioned Box is inconsistent in when it calls the allocator or uses a dangling pointer.

---

### Comment 448 by Amanieu
_2023-06-28T01:21:22Z_

I've addressed the issues with ZST handling in `Box` and `Vec` in #113113.

As a side note, all the logic for handling zero-sized allocations could also be implemented as a type that wraps an `Allocator` and causes zero-sized allocations to become no-ops. We don't have to provide this in the standard library, it can just be a crate.

---

### Comment 449 by lilith
_2023-06-28T02:39:02Z_

How would such a wrapper type handle dealloc? An inconsistent abstraction
could be a footgun.


---

### Comment 450 by Amanieu
_2023-06-28T02:40:15Z_

The wrapper would return `NonNull::dangling` for all zero-sized allocations and ignore all zero-sized deallocations. Non-zero allocations are passed on to the underlying allocator.

---

### Comment 451 by lilith
_2023-06-28T02:42:39Z_

That makes sense; I forgot size was passed to dealloc.

On Tue, Jun 27, 2023, 8:40 PM Amanieu ***@***.***> wrote:

> The wrapper would return NonNull::dangling for all zero-sized allocations
> and ignore all zero-sized deallocations. Non-zero allocations are passed on
> to the underlying allocator.
>
> —
> Reply to this email directly, view it on GitHub
> <https://github.com/rust-lang/rust/issues/32838#issuecomment-1610574584>,
> or unsubscribe
> <https://github.com/notifications/unsubscribe-auth/AAA2LH4DZ5YKLOAMLJVIJGLXNOKR5ANCNFSM4CANQUGQ>
> .
> You are receiving this because you are subscribed to this thread.Message
> ID: ***@***.***>
>


---

### Comment 452 by thomcc
_2023-06-28T07:43:33Z_

> As a side note, all the logic for handling zero-sized allocations could also be implemented as a type that wraps an Allocator and causes zero-sized allocations to become no-ops.

This is true, but you end up with code performing the same checks many times. For example, both sides of the interface end up checking here if your underlying allocator doesn't handle zero-sized allocations well, and most (or at least *very many*) designs for allocators do not, in my experience.

Given that forbidding it is consistent with the earlier design (GlobalAlloc), I don't really see what we get by allowing it at this point when it clearly causes trouble both for users and implementers of the API.

---

### Comment 453 by RalfJung
_2023-06-28T11:05:03Z_

> I don't really see what we get by allowing it at this point when it clearly causes trouble both for users and implementers of the API.

It can't possible cause trouble for users -- every user that is correct wrt the more restricted API (that is UB on zero-sized allocs) is also correct wrt the current API.

---

### Comment 454 by Jules-Bertholet
_2023-06-28T12:42:59Z_

Another reason to have the collection check for zero size: for e.g. `Box<T>`, the collection knows statically whether T is a ZST, so the checks can be optimized out.

> It can't possible cause trouble for users -- every user that is correct wrt the more restricted API (that is UB on zero-sized allocs) is also correct wrt the current API.

Correctness trouble is the worst kind of trouble, but not the only kind. Users also want performance, which the current API pessimizes.

---

### Comment 455 by thomcc
_2023-06-28T14:25:36Z_

I also don't necessarily agree that the current API avoids correctness issues. We clearly had one in the stdlib code, which would have almost certainly been avoided if we made it clear that zero-sized allocations were forbidden (especially if we forbid it by making things take a NonZeroLayout or such).

My experience is that the sort of optimizations that the stdlib does here are quite common, and not a special case. This is likely because of how long we've documented that zero-sized allocations are free and infallible (something which definitely won't be universally true if we delegate this to the underlying allocator).

---

### Comment 456 by RalfJung
_2023-06-28T15:22:01Z_

FWIW, if I read https://github.com/rust-lang/rust/commit/56cbf2f22aeb6448acd7eb49e9b2554c80bdbf79 correctly, then RawVec actually did this correctly when the `Allocator` (`AllocRef` back then) trait was changed to allow zero-sized allocations. The bug was introduced in https://github.com/rust-lang/rust/commit/2526accdd35c564eee80b6453a0b4965e6a76afd, a later commit in the same PR (https://github.com/rust-lang/rust/pull/70362).

> I also don't necessarily agree that the current API avoids correctness issues.

I agree that for data structures that have a special zero-size state themselves where they don't own any memory that must be given back to the allocator, the current API does not help at all.

The question is, is that case common enough to justify hurting the alternative case where data structures would be completely fine with always owning some allocated memory, even when the size is 0 -- basically leaving it to allocators to make size 0 efficient, instead of having to implement that optimization for each data structure? Naively it seems much better to do this in the allocator once rather than in every single data structure, but clearly `Vec` disagrees since it doesn't trust the allocator doing this right. Interesting.

Here is a possible alternative to just making `Allocator` require non-ZST: we could say that passing the result of `NonNull::dangling()` to `deallocate`/`grow`/`shrink` with a size-zero layout must always be allowed and not attempt to actually deallocate that pointer. Then `Vec::new` could still use `NonNull::dangling`, but all the rest of the `Vec` code could freely treat the backing buffer as if it was created by the allocator, and `drop` could unconditionally call `deallocate`.

That would make the buggy `shrink` actually legal. It would avoid having to re-implement the "size 0 optimization" in each and every data structure, instead having it implemented once in the allocator. So just from a code structure perspective, it seems to me like that is the API that types like `Vec` actually want: a `const`-compatible, zero-cost `fn new` without having to worry about size 0 anywhere. `allocate` would be safe without the need for a `NonZeroLayout`.

What I don't know is whether this API is something allocators can reasonably implement. @thomcc what do you think?

---

### Comment 457 by nbdd0121
_2023-06-28T16:12:24Z_

> Here is a possible alternative to just making `Allocator` require non-ZST: we could say that passing the result of `NonNull::dangling()` to `deallocate`/`grow`/`shrink` with a size-zero layout must always be allowed and not attempt to actually deallocate that pointer.

How would you tell apart `dangling()` from a valid pointer?

---

### Comment 458 by RalfJung
_2023-06-28T16:18:36Z_

`Vec` does it, so clearly it is possible. It would be up to the allocator to ensure that for size 0, it can never confuse `dangling` with a valid pointer.

---

### Comment 459 by Jules-Bertholet
_2023-06-28T16:32:35Z_

`Vec` checks for `capacity == 0` to determine whether its pointer is dangling.

---

### Comment 460 by thomcc
_2023-06-28T21:02:09Z_

> a const-compatible, zero-cost fn new without having to worry about size 0 anywhere

I'm not sure how it's const-compatible unless the allocator is a const trait/impl/something (or at least allocation is const), which doesn't seem likely to be the case most of the time (since usually allocation requires a number of operations which are problematic for const). I have to think about the rest of your comment though.

---

### Comment 461 by RalfJung
_2023-06-28T21:06:29Z_

`NonNull::dangling` is a `const fn`, which is all that is needed for `Vec::new`. (The function would stay unchanged compared to what the stdlib does right now.)

To be clear, I have no idea if this proposal makes sense. The part where the allocator, in `dealloc`, has to handle `dangling` and therefore has to ensure to never actually put a real allocation (that must be freed) of size 0 at that address is certainly somewhat suspicious. I arrived at this purely from the perspective of "what would it take for `Vec` to not need to special-case capacity 0".

---

### Comment 462 by thomcc
_2023-06-28T21:10:14Z_

Oh, I see. I misinterpreted your proposal then.

I think the problem here is that now `dealloc` can't solely use allocation size to tell the difference between, since it needs to know if the allocation came from a call to alloc with a zero-sized layout, or if it came from `NonNull::dangling()`, which can return a valid pointer (and in practice often will on 32 bit targets if the alignment is sufficiently high).

---

### Comment 463 by RalfJung
_2023-06-29T06:04:12Z_

So sounds like allocators would basically be forced to not have an actual allocation for size 0, so that they can make 'deallocate' always a NOP when the size is 0.

---

### Comment 464 by thomcc
_2023-06-29T07:36:42Z_

Yeah. We'd have to document that a zero sized allocation needs to be equivalent to dangling (or at least some kind of no-op), which seems a bit odd to me, but it would work. Basically, instead of telling users of Allocator that they cant' give allocators a zero-sized layout and should allocate a zero-sized layout in a certain way, we're saying they must behave a specific manner when given zero-sized layouts.

The downside here is that the checks couldn't always be removed in cases where Allocator is a trait object. It also plausibly adds a branch into the allocation path that could otherwise be avoided. It also is a bit error-prone if not documented properly, as I go over in https://github.com/rust-lang/rust/issues/32838#issuecomment-1585684243 (although I've softened on this proposal since the issues I hit could possibly be addressed with documentation).

This would make the "handle zero-sized layout by rounding up" approach suggested elsewhere in this thread invalid though (but I don't see a way to keep it without many other downsides).

---

### Comment 465 by thomcc
_2023-08-07T03:09:18Z_

I wrote a blog post announcing that I'm intending on working on the `Allocator` design, and I wrote down (roughly) a set of things I'm thinking about <https://shift.click/blog/allocator-trait-talk>.

Largely speaking my feeling that `Allocator` needs more comprehensive rework/consideration is why I haven't filed a PR for the extra parameter for `grow`, for any changes around zero-sized allocations[^1], etc.

[^1]: I've started to have second thoughts on zero-sized allocations, which is one of the things I'm hoping to work through. I think that perhaps `Allocator`s which use resources for zero-sized allocations is a little analogous to `Iterator`s which return `None` in the middle -- IOW, could an approach more similar to `FusedIterator`/`Fuse` work? I'm not sure, maybe.

Anyway, all of this is tricky because `Allocator` is trying to serve so many roles, so it's hard to find a design that doesn't end up making a trade-off that negatively impacts something or other, and it takes a lot of experimentation to play around with different implementations of the trait and code using it.

---

### Comment 466 by wallgeek
_2024-01-23T18:47:52Z_

I'm sorry but isn't both "new_in" and "with_capacity_in" have very minor mistakes in source documentation in example section? Or am I missing something?
https://doc.rust-lang.org/std/collections/struct.VecDeque.html

---

### Comment 467 by udoprog
_2024-01-23T19:26:39Z_

@wallgeek No, you're right. They don't exemplify the APIs at all. It should be fixed!

---

### Comment 468 by pravic
_2024-01-24T07:14:35Z_

I've checked all `new_in` methods in https://doc.rust-lang.org/std/index.html?search=new_in&filter-crate=std:

* [VecDeque](https://doc.rust-lang.org/std/collections/struct.VecDeque.html#method.new_in) - a blunt copy from `new`
* [BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html#method.new_in)
  - > Makes a new empty BTreeMap with a reasonable choice for B.
  - what is `B` exactly?
* [BTreeSet](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html#method.new_in) - ditto with `BTreeMap`
* [LinkedList](https://doc.rust-lang.org/std/collections/struct.LinkedList.html#method.new_in)
  - > Constructs an empty `LinkedList<T, A>`.
  - the description can be improved

The rest is okay.

---

### Comment 469 by SimonSapin
_2024-01-24T21:16:33Z_

> * what is `B` exactly?

Currently in std:

```rust
const B: usize = 6;
```

The struct-level docs explain this and compare a B-tree with a binary tree that would have individual allocations for each item: 

https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
> A B-Tree instead makes each node contain B-1 to 2B-1 elements in a contiguous array. By doing this, we reduce the number of allocations by a factor of B, and improve cache efficiency in searches. 

It’s probably not relevant for the docs of a constructor to talk about the "choice" of B, since that choice is compile-time constant in the current implementation. (As opposed to something users could influence like `Vec::with_capacity`)

---

### Comment 470 by vmolsa
_2024-10-31T04:20:51Z_

With allocator_api, we now have safe memory allocation methods like try_new() for types like Box and Arc, and some collections (Vec, HashMap) support try_reserve as a workaround. However, collections like BTreeMap lack equivalent allocation-checked methods for operations like insert. To build a more consistent API, could we introduce allocation-checked variants across std::collections::* with a clear prefix like checked_*, or safe_*, etc.. (e.g., safe_insert, checked_push)? These methods would return Result<T, AllocError> on allocation failure, streamlining safe memory allocation handling.

---

### Comment 471 by Sewer56
_2024-12-05T22:28:47Z_

[I added a small guide](https://reloaded-project.github.io/Reloaded-III/Research/Demystifying-Rust-Allocators.html) for current state of `allocator_api` into one of my projects' documentation. Search engines don't seem to handle it, but it may be useful for some new people looking around.

In any case, some general feedback on the current state from own experiences is below.

## General Purpose Feedback

When trying `allocator_api` for the first time a while back, while I was still relatively new to Rust, I found it to be a tiny bit challenging to use. 

The semantics weren't immediately clear at first, e.g. 
- How to consume an allocator.
- Allocator reference vs zero sized allocator (2 ways to design an allocator)
- Overheads (if any) on the heap

Even with a blog post or two around, not everything was clear, so I added another resource (above).
Think this can be resolved with just a tiny bit more examples/docs.

## Allocate & Remaining Methods

One thing I also found a bit weird at first is `allocate` returns a `NonNull<[u8]>`, but the other APIs take a `NonNull<u8>`.

It may not be immediately clear to people newer to Rust that the representation of a slice in Rust is `ptr + len` (fat pointer), there is (technically) a possibility that someone may think the representation is a pointer to `len + data` in same memory allocation; and that assumption may confuse a user.

I've temporarily been in that camp, until I learned that `[T]` is actually unsized ([Dynamically sized type](https://doc.rust-lang.org/book/ch19-04-advanced-types.html#dynamically-sized-types-and-the-sized-trait) (DST)), and references to the data is always `ptr + len`. The magic is the phrase `[Pointer types](https://doc.rust-lang.org/reference/types/pointer.html) to DSTs are sized but have twice the size of pointers to sized types` from the [DST Docs](https://doc.rust-lang.org/reference/dynamically-sized-types.html)

Providing a blanket implementation here may be useful, so user can just pass whatever they received from `allocate` to the other functions.

## allocator_api2

For now we have `allocator_api2` to provide re-exports. 

It's generally not so hard to use, however some 'best practices' could be noted somewhere, given how long actual design of `allocator_api` has been taking; these things that come to mind:

- There's no coercion to unsized types without the actual std, type so you have to rely on undocumented `unsize_box` hack. (and similar caveats)
- You have to write `no_std` to avoid `std` prelude (can still use `std` crate via `extern crate`), otherwise it's easy to mix types such std `Vec` and `allocator_api2` Vec.
- Code duplication and cache (in-efficiency), since programs compiled will now have multiple copies of the regular containers. 

-------------

In any case, since the thread has died down, for over a year, does anyone know the future plans/state for `allocator_api`?

There's a lot of talk above, as usual; but it's hard to make conclusions given the long passage of time since the thread was alive, conversations may have been happening in the working group chats for example; so I figured I'd ask.

---

### Comment 472 by v-thakkar
_2025-02-10T06:32:43Z_

@Sewer56 Thanks for the summary and your small guide. It's super useful. There have been some discussions happening related to this on Rust Zulip. https://rust-lang.zulipchat.com/#narrow/channel/219381-t-libs/topic/no_global_oom_handling.20removal/near/498629183

---

### Comment 473 by abgros
_2025-03-28T06:09:59Z_

As someone who just finished up implementing `Allocator` for my new crate (https://crates.io/crates/stalloc), I think I'm qualified enough to comment on this API.

And... it's pretty bad. Many of the design decisions have forced me to make performance sacrifices for no reason:
* `allocate()` should absolutely *not* allow zero-sized allocations. In my code, this ends up being special-cased to return a dangling pointer, resulting in additional overhead on every single allocation.
* `deallocate()` also needs a special case to handle user code trying to deallocate a dangling pointer.
* `grow()`: this one is horrendous. It stuffs together four separate tasks into one function: trying to grow, allocating, copying, freeing. That should be four separate function calls! This questionable design decision means that growing a `Vec` often results in the allocator pointlessly copying uninitialized memory because it doesn't realize that we only care about the parts up to `len`. Not only that, you're also allowed to pointlessly call it with `old_layout == new_layout`, which has to be special cased. And not only that: you're allowed to call it with a *different alignment* than your original allocation (why...?) which has to be, once again, checked for and special cased. And *not only that*, you're also allowed to call this function with a dangling pointer, in case you want to "grow" your zero-size allocation! My proposal is to kill this one with fire. As an alternative, I would like to propose [`grow_in_place()`](https://docs.rs/stalloc/latest/stalloc/struct.Stalloc.html#method.grow_in_place) with the signature:
```rs
/// Tries to grow an allocation in-place. If that isn’t possible, this function is a no-op.
/// # Safety
///
/// `ptr` must point to a valid allocation of `old_layout.size()` bytes, which was
/// originally allocated with an alignment of `old_layout.align()`, and `new_size > old_layout.size()`.
pub unsafe fn grow_in_place(
    &self,
    ptr: NonNull<u8>,
    old_layout: Layout,
    new_size: usize,
) -> Result<NonNull<[u8]>, AllocError>;
```
If a user calls this function and gets `AllocError`, they can decide whether they want to reallocate, copy, and free — it's not foisted upon them.
* `shrink()`: This one has all the same pitfalls and special cases as `grow()`. Implementing `Allocator` is so tricky and exhausting because you have to handle absurd situations like user code allocating 1 byte, "shrinking" it to 1 byte (with a higher alignment — time to reallocate!), shrinking it to zero bytes (wait, isn't that just `deallocate()`...?), shrinking it *again* to zero bytes but with a higher alignment (wait, is it bad if the dangling pointer isn't aligned enough...?), and then "growing" it to zero bytes with an even higher alignment (you get the picture). Also, what happens if the user calls `shrink()` with a size of `0` and an alignment of `2^29` and the resulting "dangling" pointer happens to end up right in the middle of your allocator? Admittedly, I'm pretty sure that would never happen, but hopefully you weren't about to try identifying a dangling pointer at runtime. Anyway, to match `grow_in_place`, I propose replacing `shrink()` with [`shrink_in_place`](https://docs.rs/stalloc/latest/stalloc/struct.Stalloc.html#method.shrink_in_place):
```rs
/// Shrinks the allocation. This function always succeeds and never reallocates.
/// # Safety
///
/// `ptr` must point to a valid allocation of `old_layout.size()` bytes, which was originally
/// allocated with an alignment of `old_layout.align()`, and `new_size` must be in `1..old_layout.size()`.
pub unsafe fn shrink_in_place(
    &self,
    ptr: NonNull<u8>,
    old_layout: Layout,
    new_size: usize,
) -> NonNull<[u8]>
```

In general, the `Allocator` methods should be *as unsafe as possible*, because it's easy to create a wrapper with extra safety checks, but doing the reverse is impossible.

On an unrelated note, we really need `String::new_in()`. That might be for another thread, though...

---

### Comment 474 by abgros
_2025-03-28T06:39:04Z_

Something else I thought of: it might be useful to have a method like `grow_up_to()`, which tries to grow an allocation as much as it can, even if it can't completely fulfill the request:
```rs
/// Tries growing an allocation in-place. If that isn't possible, the allocator will grow
/// as much as it is able to. This function always succeeds and never reallocates.
/// # Safety
///
/// `ptr` must point to a valid allocation of `old_layout.size()` bytes, which was
/// originally allocated with an alignment of `old_layout.align()`, and `new_size > old_layout.size()`.
unsafe fn grow_up_to(
    &self,
    ptr: NonNull<u8>,
    old_layout: Layout,
    new_size: usize,
) -> NonNull<[u8]>
```
But I'm not sure whether the stdlib collection types would be able to use this as an optimization.

---

### Comment 475 by programmerjake
_2025-03-28T06:50:56Z_

> * `grow()`: this one is horrendous. It stuffs together four separate tasks into one function: trying to grow, allocating, copying, freeing.

we need the semantics to be such that you can implement it by calling C's `realloc`, hence why it does all the reallocating/copying/freeing if it can't resize in place.

---

### Comment 476 by abgros
_2025-03-28T07:11:00Z_

> > * `grow()`: this one is horrendous. It stuffs together four separate tasks into one function: trying to grow, allocating, copying, freeing.
> 
> we need the semantics to be such that you can implement it by calling C's `realloc`, hence why it does all the reallocating/copying/freeing if it can't resize in place.

In principle, I guess you _could_ implement `grow_in_place()` with C's `realloc`. It might end up moving and copying your data against your wishes, but user code should have no way of finding this out (the old pointer is always invalidated).

---

### Comment 477 by Amanieu
_2025-03-28T11:31:00Z_

For the case of zero-sized allocations, the intent is that the allocator is not required to special case it to avoid allocating memory. Instead it is the caller's job to avoid invoking the allocator in the first place when it wants to avoid allocating memory for zero-sized allocations (for example, `Vec` does this). It's therefore perfectly fine for the allocator to implement ZST allocation in an inefficient manner. Admittedly the `Allocator` trait documentation could be improved to make this clearer.

> Also, what happens if the user calls shrink() with a size of 0 and an alignment of 2^29 and the resulting "dangling" pointer happens to end up right in the middle of your allocator?

Zero-sized "allocations" are allowed to be in the middle of other allocations and aren't considered to overlap them. See https://github.com/rust-lang/reference/pull/1657 which discusses this.

---

### Comment 478 by RalfJung
_2025-03-28T12:56:25Z_

> Zero-sized "allocations" are allowed to be in the middle of other allocations and aren't considered to overlap them. See https://github.com/rust-lang/reference/pull/1657 which discusses this.

I don't think we reached a consensus on this question there, and it is certainly not reflected in the PR.

So no, until we have explicitly decided otherwise, zero-sized allocations cannot be in the middle of other allocations, not for the purpose of the Abstract Machine. That said, the `Allocator` trait does not have to return AM allocations, it just has to return pointers that satisfy certain properties, so the question for `Allocator` implementations is somewhat independent.

---

### Comment 479 by Jules-Bertholet
_2025-03-28T13:32:16Z_

> you're allowed to call it with a _different alignment_ than your original allocation (why...?)

It’s not the common case, but I’ve written code that makes use of this (in https://github.com/Jules-Bertholet/unsized-vec/).

---

### Comment 480 by Keith-Cancel
_2025-04-07T07:00:26Z_

When discussing some things on Zulip. One of the things I saw was the desire for Allocators to (work with/be part of) the proposed store API so that the store API does not make the Allocator API redundant in a lot of cases.  So to make this api forward compatibility to allow to be stabilized while the details of the store API or worked on I came up with this:
https://github.com/Keith-Cancel/allocator_api_forward_compat


---

### Comment 481 by petertodd
_2025-04-08T11:02:09Z_

@abgros re: realloc/grow, I don't know if any allocation libraries actually do this in the wild. But for larger allocations allocators may be implementing realloc more efficiently by remapping the original pages to new virtual memory locations using the OS's virtual memory API (e.g. [mremap(2)](https://man7.org/linux/man-pages/man2/mremap.2.html) on linux). In this case, realloc/grow would still need to return a new pointer. But the actual physical memory pages wouldn't have been actually copied.

Obviously, this is only worth it for large allocations as changing virtual memory mappings has a cost.

---

### Comment 482 by mikeyhew
_2025-06-11T21:07:33Z_

One thing that's a bit sad about the current allocator api is it requires `Box<T, A>` to store `A` alongside the pointer in order to deallocate it later. For bump allocators like bumpalo this means `Box<T, &Bump>` takes up an extra 8 bytes, even though deallocation is a no-op. Meanwhile the `Box` type that comes with `bumpalo` just contains a single pointer. Has any thought been put into changing the allocator api to avoid this overhead?

```rust
#![feature(allocator_api)]

use bumpalo::{Bump, boxed::Box as BumpaloBox};

fn main() {
    dbg!(std::mem::size_of::<Box<i32>>());
    dbg!(std::mem::size_of::<Box<i32, &Bump>>());
    dbg!(std::mem::size_of::<BumpaloBox<i32>>());
}
```

```
[crates/allocator_playground/src/main.rs:6:5] std::mem::size_of::<Box<i32>>() = 8
[crates/allocator_playground/src/main.rs:7:5] std::mem::size_of::<Box<i32, &Bump>>() = 16
[crates/allocator_playground/src/main.rs:8:5] std::mem::size_of::<BumpaloBox<i32>>() = 8
```

---

### Comment 483 by Amanieu
_2025-06-11T21:56:08Z_

If your type doesn't require dropping then you don't need to use `Box` at all when using bumpalo, you can just use a `&'a mut T` where `'a` is the lifetime of the allocator. Otherwise I think it's fine to just use bumpalo's provided `Box` type for this.

---

### Comment 484 by programmerjake
_2025-06-11T22:52:11Z_

> One thing that's a bit sad about the current allocator api is it requires `Box<T, A>` to store `A` alongside the pointer in order to deallocate it later. For bump allocators like bumpalo this means `Box<T, &Bump>` takes up an extra 8 bytes, even though deallocation is a no-op. Meanwhile the `Box` type that comes with `bumpalo` just contains a single pointer. Has any thought been put into changing the allocator api to avoid this overhead?

As mentioned [here](https://github.com/rust-lang/rfcs/pull/3470/commits/849585bfe8552db4f06d397ae8c59df859690916), I came up with an idea of having `Box<T, D>` where `D: BoxDrop` which takes the `Box` by value when it's dropped: [proposal](https://github.com/rust-lang/rfcs/pull/3470#issuecomment-1674249638), [sample usage](https://github.com/rust-lang/rfcs/pull/3470#issuecomment-1674265515)

`bumpalo` could use a zero-sized type for the `BoxDrop`, and just have it call `drop_in_place` and forget the `Box`.

---

### Comment 485 by mikeyhew
_2025-06-11T23:22:02Z_

I just realized there's another repo where comments like this are supposed to go, and there's already an open issue that seems pretty related (https://github.com/rust-lang/wg-allocators/issues/112) so I'll comment there.

---

### Comment 486 by FeldrinH
_2025-06-11T23:43:43Z_

> Otherwise I think it's fine to just use bumpalo's provided `Box` type for this.

For better interop, the bumpalo Box could be defined as a std Box with a noop allocator. Though this will make allocating the box a little clunkier, because you can't just use the standard Box::new_in.

---

### Comment 487 by petertodd
_2025-06-17T10:13:11Z_

Bumpalo would benefit if deallocation was done via a handle, allowing the
handle to be an empty type:

```
trait Alloc {
	type Dealloc : Dealloc;
}

struct Box<T, A: Alloc = Global> {
	ptr: NonNull<T>,
	dealloc: A::Dealloc;
}
```


---

### Comment 488 by jpochyla
_2025-07-17T18:54:14Z_

> If your type doesn't require dropping then you don't need to use Box at all when using bumpalo, you can just use a &'a mut T where 'a is the lifetime of the allocator. Otherwise I think it's fine to just use bumpalo's provided Box type for this.

But this approach is extremely cumbersome when developing higher level data structures (i.e. trees) that are generic over `A`.

---

### Comment 489 by cosmicexplorer
_2026-01-02T18:31:05Z_

I have little to say regarding the effectiveness of the current API.

# `alloc::vec` is much more common

However, I would like to note that stabilizing a trait -- *any* trait -- makes it possible for people to build code using `alloc::vec` and `alloc::collections`, which is significantly more common and more useful than the somewhat niche use case of implementing a custom data structure which can use a custom allocator by directly calling the methods.

In particular, even if the `Allocator` trait *were* to deprecate methods (as C++ did) or add new ones, the standard use cases that deploy `alloc::{vec,collections}` alone would *not* have to change. I think this is an extremely underappreciated nuance of stabilizing the trait as-is.

## case study: `no_std` zstd
I am currently implementing yet another zstd implementation and would very much like to make it `no_std` on stable (hence this reply). At the moment I believe I will literally only require `alloc::vec::Vec` to implement the core codec logic. Without `alloc::Allocator`, I will probably have to define a crate-specific trait for growable buffers, which requires the client to implement a lot of logic I *should* be able to do in my own crate with `alloc::vec`.

# poor interaction with `cfg(...)` logic

Due to an unfortunate hole in Rust syntax, while it is possible to apply `#[cfg(...)]` attributes on generic parameter declarations in an item, it is *not* possible to apply them to usages of those parameters. This means that while the `Allocator` trait remains unstable, it is incredibly difficult to avoid duplicating massive amounts of code. To make this concrete:

```rust
// This is legal:
struct A<#[cfg(feature = "nightly")] B>(#[cfg(feature = "nightly")] B);

// impl<#[cfg(feature = "nightly")] B> is allowed,
// but A<#[cfg(feature = "nightly")] B> is a parse error:
impl<#[cfg(feature = "nightly")] B> A<#[cfg(feature = "nightly")] B> {
  ...
}
```

**In particular, one very useful crate I would like to use with `no_std` and parameterized allocators is `smallvec`.** I have done so here: https://github.com/servo/rust-smallvec/compare/v2...cosmicexplorer:rust-smallvec:parameterized-allocator?expand=1#diff-b1a35a68f14e696205874893c07fd24fdb88882b47c23cc0e0c80a30c7d53759 *(preview isn't working; go to "files changed" and then `src/lib.rs`. The direct file contents are here: https://github.com/cosmicexplorer/rust-smallvec/blob/7e56901adca52e0a60431766e0b8c25fdb09eaf0/src/lib.rs#L480).*

It makes use of a very very complex proc macro that works around this issue by identifying usages of generic params, then generating 2^n instances of the item (e.g. an `impl`) for `n` distinct `#[cfg(...)]`-constrained generic params. What this means is that the `smallvec` with parameterized allocator can do this:
```rust
#[conditional_impl_type_bounds]
unsafe impl<T: Send, #[cfg(feature = "allocator-api")] A: Allocator + Send, const N: usize> Send
    for SmallVec<T, N, A>
{
}
```

which gets converted to this:
```rust
#[cfg(feature = "allocator-api")]
unsafe impl<T: Send, A: Allocator + Send, const N: usize> Send
    for SmallVec<T, N, A>
{
}
#[cfg(not(feature = "allocator-api"))]
unsafe impl<T: Send, const N: usize> Send
    for SmallVec<T, N>
{
}
```

I don't really think this is a good approach to do via macros in general, and it quickly requires reimplementing much of the compiler's internal logic to cover all cases. I think it would be useful to eventually patch this hole in the syntax, but I don't want to have to wrestle over changes to the parser, which would be far more breaking than the changes we may need to perform to make the `Allocator` trait work perfectly as per the lengthy discussion in this thread.

The specific difficulty in applying `cfg(...)` constraints to trait impls in Rust at the moment is to me a further argument to stabilize this trait, because it introduces much more code churn for this trait to be unstable than for other unstable features.

# reiterating: the `alloc::vec` use case is very common
This trait's instability holds back a ridiculous number of use cases I would like to create very small libraries with Rust for. The standard library is not necessary for a huge amount of logic, in particular codec implementations like zstd or av1 which largely need to manage buffers of bytes. This is also the kind of logic that is particularly easy to compile to wasm. This is one of the reasons along with coroutines that I have considered using C++ for when looking to produce a regex engine.

I understand that domain experts have very strong and well-educated opinions about the propriety of the current API. I would like to propose that methods be added and/or deprecated in follow-up work instead of further blocking `alloc::vec` for `no_std` crates.

---

## Timeline Events

- **labeled** by **nikomatsakis** at 2016-04-08T20:56:09Z
- **labeled** by **nikomatsakis** at 2016-04-08T20:56:09Z
- **labeled** by **nikomatsakis** at 2016-04-08T20:56:09Z
- **labeled** by **nikomatsakis** at 2016-04-08T20:56:09Z
- **cross-referenced** by **nikomatsakis** at 2016-04-08T20:56:44Z
- **commented** by **gereeter** at 2016-04-11T03:07:19Z
- **commented** by **gereeter** at 2016-04-11T03:12:08Z
- **cross-referenced** by **pnkfelix** at 2016-10-12T16:53:15Z
- **referenced** by **eddyb** at 2016-10-18T21:53:21Z
- **referenced** by **eddyb** at 2016-10-19T04:33:32Z
- **referenced** by **eddyb** at 2016-10-19T05:00:00Z
- **commented** by **pnkfelix** at 2016-10-26T13:04:27Z
- **commented** by **pnkfelix** at 2016-10-26T13:05:05Z
- **mentioned** by **gereeter** at 2016-10-26T13:05:05Z
- **subscribed** by **gereeter** at 2016-10-26T13:05:05Z
- **commented** by **pnkfelix** at 2016-10-31T17:38:52Z
- **cross-referenced** by **Ixrec** at 2016-12-10T22:05:30Z
- **commented** by **joshlf** at 2017-01-04T20:12:58Z
- **commented** by **Ericson2314** at 2017-01-04T20:22:53Z
- **commented** by **nikomatsakis** at 2017-01-04T20:25:22Z
- **mentioned** by **pnkfelix** at 2017-01-04T20:25:22Z
- **subscribed** by **pnkfelix** at 2017-01-04T20:25:22Z
- **commented** by **joshlf** at 2017-01-04T20:27:20Z
- **mentioned** by **Ericson2314** at 2017-01-04T20:27:20Z
- **subscribed** by **Ericson2314** at 2017-01-04T20:27:20Z
- **commented** by **steveklabnik** at 2017-01-04T20:42:32Z
- **mentioned** by **joshlf** at 2017-01-04T20:42:32Z
- **subscribed** by **joshlf** at 2017-01-04T20:42:32Z
- **commented** by **Ericson2314** at 2017-01-04T21:01:36Z
- **mentioned** by **joshlf** at 2017-01-04T21:01:36Z
- **subscribed** by **joshlf** at 2017-01-04T21:01:36Z
- **mentioned** by **steveklabnik** at 2017-01-04T21:01:36Z
- **subscribed** by **steveklabnik** at 2017-01-04T21:01:36Z
- **commented** by **joshlf** at 2017-01-04T21:27:36Z
- **mentioned** by **Ericson2314** at 2017-01-04T21:27:36Z
- **subscribed** by **Ericson2314** at 2017-01-04T21:27:36Z
- **mentioned** by **steveklabnik** at 2017-01-04T21:27:36Z
- **subscribed** by **steveklabnik** at 2017-01-04T21:27:36Z
- **commented** by **alexreg** at 2017-01-04T21:54:03Z
- **mentioned** by **joshlf** at 2017-01-04T21:54:03Z
- **subscribed** by **joshlf** at 2017-01-04T21:54:03Z
- **commented** by **joshlf** at 2017-01-04T21:58:58Z
- **commented** by **Ericson2314** at 2017-01-04T22:01:33Z
- **commented** by **alexreg** at 2017-01-04T22:02:54Z
- **commented** by **alexreg** at 2017-01-04T22:03:49Z
- **commented** by **joshlf** at 2017-01-04T22:13:00Z
- **mentioned** by **Ericson2314** at 2017-01-04T22:13:00Z
- **subscribed** by **Ericson2314** at 2017-01-04T22:13:00Z
- **commented** by **alexreg** at 2017-01-04T22:16:15Z
- **mentioned** by **Ericson2314** at 2017-01-04T22:16:16Z
- **subscribed** by **Ericson2314** at 2017-01-04T22:16:16Z
- **commented** by **joshlf** at 2017-01-04T22:28:41Z
- **commented** by **alexreg** at 2017-01-05T00:16:00Z
- **commented** by **joshlf** at 2017-01-05T00:53:18Z
- **commented** by **alexreg** at 2017-01-05T00:55:05Z
- **cross-referenced** by **burdges** at 2017-01-12T06:55:56Z
- **cross-referenced** by **burdges** at 2017-01-12T07:02:05Z
- **commented** by **burdges** at 2017-01-12T18:13:02Z
- **referenced** by **hawkw** at 2017-02-03T19:12:47Z
- **commented** by **hawkw** at 2017-02-07T17:35:14Z
- **commented** by **Ericson2314** at 2017-03-20T19:59:46Z
- **cross-referenced** by **chriskrycho** at 2017-03-29T22:07:24Z
- **commented** by **joshlf** at 2017-04-05T23:15:35Z
- **commented** by **eddyb** at 2017-04-06T15:20:24Z
- **mentioned** by **joshlf** at 2017-04-06T15:20:24Z
- **subscribed** by **joshlf** at 2017-04-06T15:20:24Z
- **commented** by **joshlf** at 2017-04-06T16:17:36Z
- **commented** by **joshlf** at 2017-04-06T16:21:08Z
- **commented** by **Ericson2314** at 2017-04-06T17:49:19Z
- **commented** by **Zoxc** at 2017-04-06T18:14:37Z
- **commented** by **joshlf** at 2017-04-06T20:30:49Z
- **mentioned** by **Zoxc** at 2017-04-06T20:30:49Z
- **subscribed** by **Zoxc** at 2017-04-06T20:30:49Z
- **mentioned** by **Ericson2314** at 2017-04-06T20:30:49Z
- **subscribed** by **Ericson2314** at 2017-04-06T20:30:49Z
- **commented** by **joshlf** at 2017-05-05T17:40:24Z
- **cross-referenced** by **Gankra** at 2017-05-11T04:43:18Z
- **cross-referenced** by **alexcrichton** at 2017-05-19T16:48:51Z
- **commented** by **alexcrichton** at 2017-05-19T16:52:25Z
- **mentioned** by **joshlf** at 2017-05-19T16:52:25Z
- **subscribed** by **joshlf** at 2017-05-19T16:52:25Z
- **commented** by **Ericson2314** at 2017-05-19T21:55:02Z
- **mentioned** by **alexcrichton** at 2017-05-19T21:55:02Z
- **subscribed** by **alexcrichton** at 2017-05-19T21:55:02Z
- **commented** by **joshlf** at 2017-05-19T22:37:18Z
- **mentioned** by **alexcrichton** at 2017-05-19T22:37:18Z
- **subscribed** by **alexcrichton** at 2017-05-19T22:37:18Z
- **commented** by **alexcrichton** at 2017-05-20T05:02:56Z
- **mentioned** by **joshlf** at 2017-05-20T05:02:56Z
- **subscribed** by **joshlf** at 2017-05-20T05:02:56Z
- **commented** by **pnkfelix** at 2017-05-30T14:35:49Z
- **mentioned** by **alexcrichton** at 2017-05-30T14:35:49Z
- **subscribed** by **alexcrichton** at 2017-05-30T14:35:49Z
- **commented** by **pnkfelix** at 2017-05-30T14:50:29Z
- **commented** by **pnkfelix** at 2017-05-30T14:58:05Z
- **mentioned** by **hawkw** at 2017-05-30T14:58:05Z
- **subscribed** by **hawkw** at 2017-05-30T14:58:05Z
- **cross-referenced** by **gereeter** at 2017-06-01T06:49:11Z
- **commented** by **joshlf** at 2017-06-06T04:00:12Z
- **mentioned** by **alexcrichton** at 2017-06-06T04:00:12Z
- **subscribed** by **alexcrichton** at 2017-06-06T04:00:12Z
- **mentioned** by **pnkfelix** at 2017-06-06T04:00:12Z
- **subscribed** by **pnkfelix** at 2017-06-06T04:00:12Z
- **commented** by **alexcrichton** at 2017-06-06T18:59:51Z
- **commented** by **joshlf** at 2017-06-06T19:13:53Z
- **mentioned** by **alexcrichton** at 2017-06-06T19:13:53Z
- **subscribed** by **alexcrichton** at 2017-06-06T19:13:53Z
- **cross-referenced** by **alexcrichton** at 2017-06-06T22:10:40Z
- **commented** by **alexcrichton** at 2017-06-06T22:21:32Z
- **mentioned** by **joshlf** at 2017-06-06T22:21:32Z
- **subscribed** by **joshlf** at 2017-06-06T22:21:32Z
- **mentioned** by **ruuda** at 2017-06-06T22:21:32Z
- **subscribed** by **ruuda** at 2017-06-06T22:21:32Z
- **commented** by **ruuda** at 2017-06-07T10:41:16Z
- **commented** by **retep998** at 2017-06-07T11:23:18Z
- **mentioned** by **ruuda** at 2017-06-07T11:23:18Z
- **subscribed** by **ruuda** at 2017-06-07T11:23:18Z
- **commented** by **pnkfelix** at 2017-06-07T15:42:17Z
- **mentioned** by **alexcrichton** at 2017-06-07T15:42:17Z
- **subscribed** by **alexcrichton** at 2017-06-07T15:42:17Z
- **commented** by **alexcrichton** at 2017-06-07T15:45:12Z
- **mentioned** by **pnkfelix** at 2017-06-07T15:45:12Z
- **subscribed** by **pnkfelix** at 2017-06-07T15:45:12Z
- **mentioned** by **ruuda** at 2017-06-20T14:36:46Z
- **subscribed** by **ruuda** at 2017-06-20T14:36:46Z
- **mentioned** by **hanna-kruppe** at 2017-06-20T14:36:46Z
- **subscribed** by **hanna-kruppe** at 2017-06-20T14:36:46Z
- **mentioned** by **Gankra** at 2017-06-20T14:36:46Z
- **subscribed** by **Gankra** at 2017-06-20T14:36:47Z
- **mentioned** by **alexcrichton** at 2017-06-20T14:36:47Z
- **subscribed** by **alexcrichton** at 2017-06-20T14:36:47Z
- **commented** by **alexcrichton** at 2017-06-20T14:37:59Z
- **renamed** by **alexcrichton** at 2017-06-20T14:38:16Z
- **commented** by **Ericson2314** at 2017-06-20T14:52:39Z
- **cross-referenced** by **alexcrichton** at 2017-06-20T15:11:06Z
- **commented** by **alexcrichton** at 2017-06-20T15:11:43Z
- **commented** by **alexcrichton** at 2017-06-20T17:22:16Z
- **mentioned** by **pnkfelix** at 2017-06-20T17:22:16Z
- **subscribed** by **pnkfelix** at 2017-06-20T17:22:16Z
- **mentioned** by **pnkfelix** at 2017-06-21T11:21:28Z
- **subscribed** by **pnkfelix** at 2017-06-21T11:21:28Z
- **cross-referenced** by **pnkfelix** at 2017-06-21T11:47:38Z
- **commented** by **pnkfelix** at 2017-06-21T11:49:42Z
- **commented** by **pnkfelix** at 2017-06-21T11:50:34Z
- **cross-referenced** by **mattico** at 2017-06-29T20:21:08Z
- **commented** by **SimonSapin** at 2017-07-08T08:33:00Z
- **commented** by **SimonSapin** at 2017-07-08T08:33:54Z
- **commented** by **eddyb** at 2017-07-08T08:38:10Z
- **mentioned** by **SimonSapin** at 2017-07-08T08:38:10Z
- **subscribed** by **SimonSapin** at 2017-07-08T08:38:10Z
- **commented** by **joshlf** at 2017-07-08T16:26:54Z
- **mentioned** by **SimonSapin** at 2017-07-08T16:26:54Z
- **subscribed** by **SimonSapin** at 2017-07-08T16:26:54Z
- **commented** by **alexcrichton** at 2017-07-08T18:27:59Z
- **mentioned** by **SimonSapin** at 2017-07-08T18:27:59Z
- **subscribed** by **SimonSapin** at 2017-07-08T18:27:59Z
- **commented** by **SimonSapin** at 2017-07-08T18:55:37Z
- **commented** by **retep998** at 2017-07-09T00:20:27Z
- **commented** by **SimonSapin** at 2017-07-09T06:24:48Z
- **cross-referenced** by **Wilfred** at 2017-07-09T21:55:24Z
- **cross-referenced** by **DavidDeSimone** at 2017-07-09T23:50:04Z
- **referenced** by **jonhoo** at 2017-07-10T21:45:08Z
- **commented** by **pnkfelix** at 2017-07-13T11:05:53Z
- **mentioned** by **alexcrichton** at 2017-07-13T11:05:53Z
- **subscribed** by **alexcrichton** at 2017-07-13T11:05:53Z
- **commented** by **SimonSapin** at 2017-07-13T11:22:47Z
- **mentioned** by **pnkfelix** at 2017-07-13T11:22:47Z
- **subscribed** by **pnkfelix** at 2017-07-13T11:22:47Z
- **commented** by **pnkfelix** at 2017-07-13T11:29:14Z
- **mentioned** by **SimonSapin** at 2017-07-13T11:29:14Z
- **subscribed** by **SimonSapin** at 2017-07-13T11:29:14Z
- **commented** by **pnkfelix** at 2017-07-13T11:31:30Z
- **commented** by **SimonSapin** at 2017-07-13T11:40:38Z
- **commented** by **pnkfelix** at 2017-07-13T11:42:57Z
- **mentioned** by **SimonSapin** at 2017-07-13T11:42:57Z
- **subscribed** by **SimonSapin** at 2017-07-13T11:42:57Z
- **commented** by **alexcrichton** at 2017-07-13T14:22:58Z
- **mentioned** by **pnkfelix** at 2017-07-13T14:22:58Z
- **subscribed** by **pnkfelix** at 2017-07-13T14:22:58Z
- **cross-referenced** by **pnkfelix** at 2017-07-13T15:19:32Z
- **labeled** by **Mark-Simulacrum** at 2017-07-22T17:54:39Z
- **cross-referenced** by **Ms2ger** at 2017-09-20T22:09:14Z
- **cross-referenced** by **alexcrichton** at 2017-10-16T17:08:57Z
- **commented** by **alexcrichton** at 2017-10-16T17:16:25Z
- **mentioned** by **rfcbot** at 2017-10-16T17:16:25Z
- **subscribed** by **rfcbot** at 2017-10-16T17:16:25Z
- **commented** by **joshlf** at 2017-10-16T17:51:18Z
- **commented** by **SimonSapin** at 2017-10-16T18:15:37Z
- **commented** by **rfcbot** at 2017-10-16T18:16:41Z
- **labeled** by **rfcbot** at 2017-10-16T18:16:41Z
- **mentioned** by **alexcrichton** at 2017-10-16T18:16:41Z
- **subscribed** by **alexcrichton** at 2017-10-16T18:16:41Z
- **mentioned** by **BurntSushi** at 2017-10-16T18:16:42Z
- **subscribed** by **BurntSushi** at 2017-10-16T18:16:42Z
- **mentioned** by **Kimundi** at 2017-10-16T18:16:42Z
- **subscribed** by **Kimundi** at 2017-10-16T18:16:42Z
- **mentioned** by **aturon** at 2017-10-16T18:16:42Z
- **subscribed** by **aturon** at 2017-10-16T18:16:42Z
- **mentioned** by **cramertj** at 2017-10-16T18:16:42Z
- **subscribed** by **cramertj** at 2017-10-16T18:16:42Z
- **mentioned** by **dtolnay** at 2017-10-16T18:16:42Z
- **subscribed** by **dtolnay** at 2017-10-16T18:16:42Z
- **mentioned** by **eddyb** at 2017-10-16T18:16:42Z
- **subscribed** by **eddyb** at 2017-10-16T18:16:42Z
- **mentioned** by **nikomatsakis** at 2017-10-16T18:16:42Z
- **subscribed** by **nikomatsakis** at 2017-10-16T18:16:42Z
- **mentioned** by **nrc** at 2017-10-16T18:16:42Z
- **subscribed** by **nrc** at 2017-10-16T18:16:42Z
- **mentioned** by **pnkfelix** at 2017-10-16T18:16:42Z
- **subscribed** by **pnkfelix** at 2017-10-16T18:16:42Z
- **mentioned** by **sfackler** at 2017-10-16T18:16:42Z
- **subscribed** by **sfackler** at 2017-10-16T18:16:42Z
- **mentioned** by **withoutboats** at 2017-10-16T18:16:42Z
- **subscribed** by **withoutboats** at 2017-10-16T18:16:42Z
- **commented** by **cramertj** at 2017-10-16T19:00:13Z
- **mentioned** by **Ericson2314** at 2017-10-16T19:00:13Z
- **subscribed** by **Ericson2314** at 2017-10-16T19:00:13Z
- **mentioned** by **joshlf** at 2017-10-16T19:02:40Z
- **subscribed** by **joshlf** at 2017-10-16T19:02:40Z
- **commented** by **alexcrichton** at 2017-10-16T19:05:14Z
- **mentioned** by **joshlf** at 2017-10-16T19:05:14Z
- **subscribed** by **joshlf** at 2017-10-16T19:05:14Z
- **mentioned** by **SimonSapin** at 2017-10-16T19:05:14Z
- **subscribed** by **SimonSapin** at 2017-10-16T19:05:14Z
- **mentioned** by **cramertj** at 2017-10-16T19:05:14Z
- **subscribed** by **cramertj** at 2017-10-16T19:05:14Z
- **commented** by **joshlf** at 2017-10-16T19:41:52Z
- **mentioned** by **cramertj** at 2017-10-16T19:41:52Z
- **subscribed** by **cramertj** at 2017-10-16T19:41:52Z
- **mentioned** by **alexcrichton** at 2017-10-16T19:41:52Z
- **subscribed** by **alexcrichton** at 2017-10-16T19:41:52Z
- **commented** by **cramertj** at 2017-10-16T20:38:13Z
- **mentioned** by **alexcrichton** at 2017-10-16T20:38:13Z
- **subscribed** by **alexcrichton** at 2017-10-16T20:38:13Z
- **commented** by **SimonSapin** at 2017-10-16T21:03:19Z
- **commented** by **SimonSapin** at 2017-10-16T21:07:44Z
- **mentioned** by **alexcrichton** at 2017-10-16T21:07:44Z
- **subscribed** by **alexcrichton** at 2017-10-16T21:07:44Z
- **commented** by **joshlf** at 2017-10-16T21:31:45Z
- **commented** by **glaebhoerl** at 2017-10-16T21:40:32Z
- **mentioned** by **cramertj** at 2017-10-16T21:40:32Z
- **subscribed** by **cramertj** at 2017-10-16T21:40:32Z
- **commented** by **cramertj** at 2017-10-17T00:05:02Z
- **mentioned** by **glaebhoerl** at 2017-10-17T00:05:02Z
- **subscribed** by **glaebhoerl** at 2017-10-17T00:05:02Z
- **commented** by **alexcrichton** at 2017-10-17T20:29:40Z
- **mentioned** by **joshlf** at 2017-10-17T20:29:40Z
- **subscribed** by **joshlf** at 2017-10-17T20:29:40Z
- **mentioned** by **cramertj** at 2017-10-17T20:29:40Z
- **subscribed** by **cramertj** at 2017-10-17T20:29:40Z
- **mentioned** by **SimonSapin** at 2017-10-17T20:29:40Z
- **subscribed** by **SimonSapin** at 2017-10-17T20:29:40Z
- **commented** by **joshlf** at 2017-10-17T20:33:18Z
- **commented** by **ruuda** at 2017-10-17T20:39:28Z
- **mentioned** by **alexcrichton** at 2017-10-17T20:39:28Z
- **subscribed** by **alexcrichton** at 2017-10-17T20:39:28Z
- **commented** by **joshlf** at 2017-10-17T20:51:17Z
- **commented** by **glaebhoerl** at 2017-10-17T23:19:57Z
- **commented** by **retep998** at 2017-10-18T03:13:40Z
- **mentioned** by **ruuda** at 2017-10-18T03:13:40Z
- **subscribed** by **ruuda** at 2017-10-18T03:13:40Z
- **commented** by **alexcrichton** at 2017-10-18T14:28:20Z
- **mentioned** by **ruuda** at 2017-10-18T14:28:20Z
- **subscribed** by **ruuda** at 2017-10-18T14:28:20Z
- **mentioned** by **joshlf** at 2017-10-18T14:28:20Z
- **subscribed** by **joshlf** at 2017-10-18T14:28:20Z
- **commented** by **ruuda** at 2017-10-18T17:44:28Z
- **commented** by **joshlf** at 2017-10-18T18:00:16Z
- **mentioned** by **alexcrichton** at 2017-10-18T18:00:16Z
- **subscribed** by **alexcrichton** at 2017-10-18T18:00:16Z
- **mentioned** by **retep998** at 2017-10-18T18:00:16Z
- **subscribed** by **retep998** at 2017-10-18T18:00:16Z
- **commented** by **retep998** at 2017-10-18T23:38:43Z
- **mentioned** by **joshlf** at 2017-10-18T23:38:43Z
- **subscribed** by **joshlf** at 2017-10-18T23:38:43Z
- **commented** by **joshlf** at 2017-10-18T23:46:55Z
- **commented** by **dtolnay** at 2017-10-19T05:39:20Z
- **mentioned** by **rfcbot** at 2017-10-19T05:39:20Z
- **subscribed** by **rfcbot** at 2017-10-19T05:39:20Z
- **commented** by **alexcrichton** at 2017-10-19T14:01:19Z
- **mentioned** by **dtolnay** at 2017-10-19T14:01:19Z
- **subscribed** by **dtolnay** at 2017-10-19T14:01:19Z
- **commented** by **Amanieu** at 2017-10-19T15:44:28Z
- **commented** by **joshlf** at 2017-10-19T20:30:31Z
- **commented** by **joshlf** at 2017-10-19T20:33:21Z
- **commented** by **cramertj** at 2017-10-19T20:36:41Z
- **commented** by **joshlf** at 2017-10-19T20:47:42Z
- **commented** by **eddyb** at 2017-10-20T11:50:03Z
- **mentioned** by **joshlf** at 2017-10-20T11:50:03Z
- **subscribed** by **joshlf** at 2017-10-20T11:50:03Z
- **commented** by **alkis** at 2017-10-24T14:32:17Z
- **commented** by **joshlf** at 2017-10-24T15:19:55Z
- **commented** by **SimonSapin** at 2017-10-24T15:35:01Z
- **commented** by **alkis** at 2017-10-24T15:49:48Z
- **mentioned** by **joshlf** at 2017-10-24T15:49:48Z
- **subscribed** by **joshlf** at 2017-10-24T15:49:48Z
- **mentioned** by **SimonSapin** at 2017-10-24T15:49:48Z
- **subscribed** by **SimonSapin** at 2017-10-24T15:49:48Z
- **commented** by **Ericson2314** at 2017-10-24T16:16:22Z
- **commented** by **joshlf** at 2017-10-24T16:41:42Z
- **mentioned** by **Ericson2314** at 2017-10-24T16:41:42Z
- **subscribed** by **Ericson2314** at 2017-10-24T16:41:42Z
- **mentioned** by **alexcrichton** at 2017-10-24T16:41:42Z
- **subscribed** by **alexcrichton** at 2017-10-24T16:41:42Z
- **commented** by **sfackler** at 2017-10-24T16:42:55Z
- **mentioned** by **joshlf** at 2017-10-24T16:42:55Z
- **subscribed** by **joshlf** at 2017-10-24T16:42:55Z
- **commented** by **joshlf** at 2017-10-24T16:49:19Z
- **commented** by **Ericson2314** at 2017-10-24T17:21:59Z
- **mentioned** by **joshlf** at 2017-10-24T17:22:30Z
- **subscribed** by **joshlf** at 2017-10-24T17:22:30Z
- **commented** by **sfackler** at 2017-10-24T17:49:04Z
- **mentioned** by **joshlf** at 2017-10-24T17:49:04Z
- **subscribed** by **joshlf** at 2017-10-24T17:49:04Z
- **commented** by **joshlf** at 2017-10-24T18:18:52Z
- **mentioned** by **sfackler** at 2017-10-24T18:18:52Z
- **subscribed** by **sfackler** at 2017-10-24T18:18:52Z
- **cross-referenced** by **SimonSapin** at 2017-10-25T08:31:56Z
- **commented** by **gnzlbg** at 2017-10-25T11:58:51Z
- **cross-referenced** by **gnzlbg** at 2017-10-25T12:04:42Z
- **commented** by **arthurprs** at 2017-10-25T14:22:39Z
- **mentioned** by **gnzlbg** at 2017-10-25T14:22:39Z
- **subscribed** by **gnzlbg** at 2017-10-25T14:22:39Z
- **commented** by **alexcrichton** at 2017-10-25T14:49:48Z
- **commented** by **bstrie** at 2017-10-25T21:47:17Z
- **commented** by **alexcrichton** at 2017-10-31T22:22:28Z
- **mentioned** by **dtolnay** at 2017-10-31T22:22:28Z
- **subscribed** by **dtolnay** at 2017-10-31T22:22:28Z
- **subscribed** by **alexcrichton** at 2017-10-31T22:22:28Z
- **subscribed** by **aturon** at 2017-10-31T22:22:28Z
- **subscribed** by **sfackler** at 2017-10-31T22:22:28Z
- **subscribed** by **BurntSushi** at 2017-10-31T22:22:28Z
- **subscribed** by **Kimundi** at 2017-10-31T22:22:28Z
- **subscribed** by **dtolnay** at 2017-10-31T22:22:28Z
- **commented** by **dtolnay** at 2017-10-31T23:20:28Z
- **commented** by **joshlf** at 2017-10-31T23:59:47Z
- **commented** by **sfackler** at 2017-11-01T00:28:19Z
- **commented** by **cramertj** at 2017-11-01T00:33:54Z
- **commented** by **sfackler** at 2017-11-01T00:42:53Z
- **commented** by **joshlf** at 2017-11-01T00:46:06Z
- **commented** by **cramertj** at 2017-11-01T00:48:25Z
- **commented** by **sfackler** at 2017-11-01T01:25:02Z
- **mentioned** by **joshlf** at 2017-11-01T01:25:02Z
- **subscribed** by **joshlf** at 2017-11-01T01:25:02Z
- **mentioned** by **cramertj** at 2017-11-01T01:25:02Z
- **subscribed** by **cramertj** at 2017-11-01T01:25:02Z
- **commented** by **sfackler** at 2017-11-01T02:23:04Z
- **commented** by **SimonSapin** at 2017-11-01T07:03:37Z
- **commented** by **alexcrichton** at 2017-11-01T15:38:07Z
- **mentioned** by **SimonSapin** at 2017-11-01T15:38:07Z
- **subscribed** by **SimonSapin** at 2017-11-01T15:38:07Z
- **commented** by **sfackler** at 2017-11-01T15:42:01Z
- **commented** by **ruuda** at 2017-11-01T20:06:55Z
- **commented** by **sfackler** at 2017-11-01T20:19:44Z
- **commented** by **Ericson2314** at 2017-11-03T21:22:20Z
- **commented** by **Ericson2314** at 2017-11-03T21:46:02Z
- **mentioned** by **sfackler** at 2017-11-03T21:46:02Z
- **subscribed** by **sfackler** at 2017-11-03T21:46:02Z
- **commented** by **sfackler** at 2017-11-03T21:52:42Z
- **mentioned** by **Ericson2314** at 2017-11-03T21:52:42Z
- **subscribed** by **Ericson2314** at 2017-11-03T21:52:42Z
- **commented** by **Ericson2314** at 2017-11-03T22:02:45Z
- **mentioned** by **sfackler** at 2017-11-03T22:02:45Z
- **subscribed** by **sfackler** at 2017-11-03T22:02:45Z
- **commented** by **sfackler** at 2017-11-03T22:11:22Z
- **commented** by **Ericson2314** at 2017-11-03T22:43:17Z
- **commented** by **sfackler** at 2017-11-03T22:48:31Z
- **commented** by **SimonSapin** at 2017-11-03T22:48:57Z
- **commented** by **Ericson2314** at 2017-11-03T22:49:26Z
- **commented** by **SimonSapin** at 2017-11-04T07:18:16Z
- **mentioned** by **Ericson2314** at 2017-11-04T07:18:16Z
- **subscribed** by **Ericson2314** at 2017-11-04T07:18:16Z
- **commented** by **Ericson2314** at 2017-11-06T20:44:02Z
- **mentioned** by **SimonSapin** at 2017-11-06T20:44:02Z
- **subscribed** by **SimonSapin** at 2017-11-06T20:44:02Z
- **commented** by **dtolnay** at 2017-11-07T21:08:33Z
- **mentioned** by **rfcbot** at 2017-11-07T21:08:33Z
- **subscribed** by **rfcbot** at 2017-11-07T21:08:33Z
- **commented** by **cramertj** at 2017-11-07T21:21:00Z
- **mentioned** by **rfcbot** at 2017-11-07T21:21:00Z
- **subscribed** by **rfcbot** at 2017-11-07T21:21:00Z
- **mentioned** by **sfackler** at 2017-11-07T21:21:00Z
- **subscribed** by **sfackler** at 2017-11-07T21:21:00Z
- **commented** by **joshlf** at 2017-11-07T21:30:00Z
- **mentioned** by **cramertj** at 2017-11-07T21:30:00Z
- **subscribed** by **cramertj** at 2017-11-07T21:30:00Z
- **commented** by **sfackler** at 2017-11-07T21:36:35Z
- **mentioned** by **joshlf** at 2017-11-07T21:36:35Z
- **subscribed** by **joshlf** at 2017-11-07T21:36:35Z
- **commented** by **joshlf** at 2017-11-07T21:41:32Z
- **commented** by **cramertj** at 2017-11-07T21:43:26Z
- **commented** by **joshlf** at 2017-11-07T21:44:59Z
- **commented** by **sfackler** at 2017-11-07T22:02:40Z
- **commented** by **cramertj** at 2017-11-07T22:05:46Z
- **mentioned** by **sfackler** at 2017-11-07T22:05:46Z
- **subscribed** by **sfackler** at 2017-11-07T22:05:46Z
- **commented** by **sfackler** at 2017-11-07T22:43:24Z
- **commented** by **joshlf** at 2017-11-07T23:00:01Z
- **commented** by **sfackler** at 2017-11-07T23:30:57Z
- **commented** by **joshlf** at 2017-11-07T23:36:41Z
- **commented** by **sfackler** at 2017-11-07T23:40:22Z
- **commented** by **joshlf** at 2017-11-07T23:49:26Z
- **commented** by **sfackler** at 2017-11-07T23:52:03Z
- **commented** by **sfackler** at 2017-11-07T23:54:11Z
- **commented** by **joshlf** at 2017-11-07T23:56:26Z
- **commented** by **sfackler** at 2017-11-07T23:59:44Z
- **commented** by **joshlf** at 2017-11-08T00:03:04Z
- **commented** by **sfackler** at 2017-11-08T00:07:23Z
- **commented** by **joshlf** at 2017-11-08T00:08:13Z
- **commented** by **sfackler** at 2017-11-08T00:11:33Z
- **commented** by **joshlf** at 2017-11-08T00:16:55Z
- **commented** by **sfackler** at 2017-11-08T00:19:15Z
- **commented** by **joshlf** at 2017-11-08T01:27:14Z
- **commented** by **sfackler** at 2017-11-08T01:35:06Z
- **commented** by **cramertj** at 2017-11-08T01:36:44Z
- **commented** by **sfackler** at 2017-11-08T01:41:33Z
- **commented** by **gnzlbg** at 2017-11-10T09:33:21Z
- **commented** by **gnzlbg** at 2017-11-10T09:42:26Z
- **commented** by **sfackler** at 2017-11-10T18:26:24Z
- **commented** by **gnzlbg** at 2017-11-13T13:02:25Z
- **commented** by **joshlf** at 2017-11-14T21:12:57Z
- **commented** by **sfackler** at 2017-11-14T21:17:30Z
- **commented** by **gnzlbg** at 2017-11-15T19:16:53Z
- **commented** by **joshlf** at 2017-11-15T19:44:23Z
- **mentioned** by **sfackler** at 2017-11-16T19:47:25Z
- **subscribed** by **sfackler** at 2017-11-16T19:47:25Z
- **commented** by **sfackler** at 2017-11-21T03:55:14Z
- **commented** by **gnzlbg** at 2017-11-21T09:29:31Z
- **commented** by **gnzlbg** at 2017-11-21T09:34:32Z
- **cross-referenced** by **SimonSapin** at 2017-11-21T15:00:49Z
- **commented** by **sfackler** at 2017-11-21T18:16:15Z
- **commented** by **gnzlbg** at 2017-11-21T22:14:53Z
- **commented** by **hanna-kruppe** at 2017-11-21T22:17:27Z
- **commented** by **gnzlbg** at 2017-11-21T22:19:31Z
- **commented** by **hanna-kruppe** at 2017-11-21T22:22:08Z
- **commented** by **gnzlbg** at 2017-11-21T22:39:34Z
- **commented** by **joshlf** at 2017-11-21T22:43:39Z
- **commented** by **sfackler** at 2017-11-21T22:48:19Z
- **mentioned** by **joshlf** at 2017-11-21T22:48:19Z
- **subscribed** by **joshlf** at 2017-11-21T22:48:20Z
- **commented** by **rfcbot** at 2017-11-21T22:48:21Z
- **labeled** by **rfcbot** at 2017-11-21T22:48:21Z
- **unlabeled** by **rfcbot** at 2017-11-21T22:48:21Z
- **commented** by **sfackler** at 2017-11-21T22:49:37Z
- **commented** by **joshlf** at 2017-11-21T23:04:40Z
- **mentioned** by **sfackler** at 2017-11-21T23:04:40Z
- **subscribed** by **sfackler** at 2017-11-21T23:04:40Z
- **commented** by **gnzlbg** at 2017-11-21T23:34:25Z
- **commented** by **sfackler** at 2017-11-21T23:49:53Z
- **commented** by **hanna-kruppe** at 2017-11-22T00:12:51Z
- **mentioned** by **gnzlbg** at 2017-11-22T00:12:51Z
- **subscribed** by **gnzlbg** at 2017-11-22T00:12:51Z
- **commented** by **gnzlbg** at 2017-11-22T10:40:44Z
- **mentioned** by **hanna-kruppe** at 2017-11-22T10:40:44Z
- **subscribed** by **hanna-kruppe** at 2017-11-22T10:40:44Z
- **mentioned** by **sfackler** at 2017-11-22T10:40:44Z
- **subscribed** by **sfackler** at 2017-11-22T10:40:44Z
- **commented** by **pnkfelix** at 2017-11-22T11:59:30Z
- **mentioned** by **alexcrichton** at 2017-11-22T11:59:30Z
- **subscribed** by **alexcrichton** at 2017-11-22T11:59:30Z
- **commented** by **pnkfelix** at 2017-11-22T12:05:33Z
- **commented** by **pnkfelix** at 2017-11-22T12:13:08Z
- **commented** by **joshlf** at 2017-11-22T15:08:25Z
- **mentioned** by **pnkfelix** at 2017-11-22T15:08:25Z
- **subscribed** by **pnkfelix** at 2017-11-22T15:08:25Z
- **commented** by **sfackler** at 2017-11-22T16:20:32Z
- **mentioned** by **gnzlbg** at 2017-11-22T16:20:32Z
- **subscribed** by **gnzlbg** at 2017-11-22T16:20:32Z
- **mentioned** by **pnkfelix** at 2017-11-22T16:20:32Z
- **subscribed** by **pnkfelix** at 2017-11-22T16:20:32Z
- **cross-referenced** by **alexcrichton** at 2017-11-24T15:29:13Z
- **commented** by **gnzlbg** at 2017-11-25T09:57:55Z
- **mentioned** by **pnkfelix** at 2017-11-25T09:57:55Z
- **subscribed** by **pnkfelix** at 2017-11-25T09:57:56Z
- **commented** by **gnzlbg** at 2017-11-29T09:56:12Z
- **mentioned** by **sfackler** at 2017-11-29T09:56:12Z
- **subscribed** by **sfackler** at 2017-11-29T09:56:12Z
- **commented** by **SimonSapin** at 2017-11-29T10:04:21Z
- **commented** by **sfackler** at 2017-11-29T17:27:08Z
- **cross-referenced** by **mbrubeck** at 2017-11-29T18:53:52Z
- **commented** by **joshlf** at 2017-11-29T22:04:32Z
- **commented** by **remexre** at 2017-11-30T03:46:48Z
- **commented** by **sfackler** at 2017-11-30T03:50:23Z
- **mentioned** by **remexre** at 2017-11-30T03:50:23Z
- **subscribed** by **remexre** at 2017-11-30T03:50:23Z
- **commented** by **remexre** at 2017-11-30T03:55:14Z
- **commented** by **gnzlbg** at 2017-11-30T08:40:34Z
- **mentioned** by **sfackler** at 2017-11-30T08:40:43Z
- **subscribed** by **sfackler** at 2017-11-30T08:40:43Z
- **commented** by **Ericson2314** at 2017-11-30T16:14:45Z
- **mentioned** by **remexre** at 2017-11-30T16:14:45Z
- **subscribed** by **remexre** at 2017-11-30T16:14:45Z
- **commented** by **jethrogb** at 2017-11-30T18:56:47Z
- **commented** by **sfackler** at 2017-11-30T19:20:33Z
- **mentioned** by **jethrogb** at 2017-11-30T19:20:33Z
- **subscribed** by **jethrogb** at 2017-11-30T19:20:33Z
- **commented** by **jethrogb** at 2017-11-30T19:58:09Z
- **mentioned** by **sfackler** at 2017-11-30T19:58:09Z
- **subscribed** by **sfackler** at 2017-11-30T19:58:09Z
- **commented** by **sfackler** at 2017-11-30T20:02:56Z
- **commented** by **SimonSapin** at 2017-11-30T22:31:18Z
- **commented** by **rfcbot** at 2017-12-01T22:58:28Z
- **commented** by **SimonSapin** at 2017-12-01T23:52:35Z
- **commented** by **sfackler** at 2017-12-02T00:30:10Z
- **commented** by **SimonSapin** at 2017-12-02T18:17:25Z
- **commented** by **SimonSapin** at 2017-12-02T18:18:04Z
- **commented** by **sfackler** at 2017-12-02T23:48:58Z
- **commented** by **SimonSapin** at 2017-12-03T08:39:42Z
- **commented** by **sfackler** at 2017-12-03T16:48:49Z
- **cross-referenced** by **sfackler** at 2017-12-04T03:03:02Z
- **commented** by **mzabaluev** at 2017-12-04T12:29:18Z
- **commented** by **sfackler** at 2017-12-04T16:57:14Z
- **commented** by **mzabaluev** at 2017-12-04T17:57:45Z
- **commented** by **joshlf** at 2017-12-04T18:48:44Z
- **mentioned** by **mzabaluev** at 2017-12-04T18:48:44Z
- **subscribed** by **mzabaluev** at 2017-12-04T18:48:44Z
- **commented** by **mzabaluev** at 2017-12-04T19:21:34Z
- **mentioned** by **joshlf** at 2017-12-04T19:21:34Z
- **subscribed** by **joshlf** at 2017-12-04T19:21:34Z
- **commented** by **joshlf** at 2017-12-04T19:29:29Z
- **commented** by **mzabaluev** at 2017-12-04T19:29:35Z
- **commented** by **SimonSapin** at 2017-12-04T22:04:05Z
- **commented** by **mzabaluev** at 2017-12-05T05:13:14Z
- **commented** by **SimonSapin** at 2017-12-05T09:06:30Z
- **commented** by **gnzlbg** at 2017-12-08T16:25:15Z
- **commented** by **Ericson2314** at 2017-12-28T03:44:25Z
- **mentioned** by **joshlf** at 2017-12-28T05:37:45Z
- **subscribed** by **joshlf** at 2017-12-28T05:37:45Z
- **mentioned** by **Gankra** at 2017-12-28T05:48:52Z
- **subscribed** by **Gankra** at 2017-12-28T05:48:52Z
- **cross-referenced** by **Ericson2314** at 2017-12-28T06:23:21Z
- **mentioned** by **eddyb** at 2017-12-28T16:22:39Z
- **subscribed** by **eddyb** at 2017-12-28T16:22:39Z
- **cross-referenced** by **Ericson2314** at 2018-01-02T16:37:44Z
- **commented** by **raphaelcohn** at 2018-01-04T18:36:09Z
- **commented** by **gnzlbg** at 2018-01-16T12:10:01Z
- **commented** by **SimonSapin** at 2018-01-16T12:41:03Z
- **mentioned** by **gnzlbg** at 2018-01-16T12:41:03Z
- **subscribed** by **gnzlbg** at 2018-01-16T12:41:03Z
- **commented** by **gnzlbg** at 2018-01-16T13:01:47Z
- **mentioned** by **alexcrichton** at 2018-01-16T13:01:47Z
- **subscribed** by **alexcrichton** at 2018-01-16T13:01:47Z
- **commented** by **SimonSapin** at 2018-01-16T13:26:42Z
- **commented** by **gnzlbg** at 2018-01-16T13:39:15Z
- **commented** by **Ericson2314** at 2018-01-16T18:23:07Z
- **commented** by **sfackler** at 2018-01-16T18:32:29Z
- **mentioned** by **Ericson2314** at 2018-01-16T18:32:29Z
- **subscribed** by **Ericson2314** at 2018-01-16T18:32:29Z
- **commented** by **SimonSapin** at 2018-01-17T07:24:04Z
- **mentioned** by **Ericson2314** at 2018-01-17T07:24:04Z
- **subscribed** by **Ericson2314** at 2018-01-17T07:24:04Z
- **commented** by **SimonSapin** at 2018-01-17T08:09:24Z
- **commented** by **gnzlbg** at 2018-01-17T10:47:45Z
- **mentioned** by **SimonSapin** at 2018-01-17T10:47:45Z
- **subscribed** by **SimonSapin** at 2018-01-17T10:47:45Z
- **mentioned** by **alexcrichton** at 2018-01-17T10:47:45Z
- **subscribed** by **alexcrichton** at 2018-01-17T10:47:45Z
- **commented** by **SimonSapin** at 2018-01-17T14:12:40Z
- **commented** by **gnzlbg** at 2018-01-17T14:18:10Z
- **commented** by **SimonSapin** at 2018-01-17T14:24:50Z
- **commented** by **gnzlbg** at 2018-01-17T14:25:45Z
- **mentioned** by **SimonSapin** at 2018-01-17T14:25:45Z
- **subscribed** by **SimonSapin** at 2018-01-17T14:25:45Z
- **commented** by **gnzlbg** at 2018-01-17T14:34:20Z
- **commented** by **SimonSapin** at 2018-01-17T14:40:39Z
- **commented** by **gnzlbg** at 2018-01-17T14:50:26Z
- **commented** by **SimonSapin** at 2018-01-17T14:55:49Z
- **commented** by **gnzlbg** at 2018-01-17T15:07:34Z
- **mentioned** by **SimonSapin** at 2018-01-17T15:07:34Z
- **subscribed** by **SimonSapin** at 2018-01-17T15:07:34Z
- **commented** by **Ericson2314** at 2018-01-17T21:11:50Z
- **mentioned** by **gnzlbg** at 2018-01-17T21:11:50Z
- **subscribed** by **gnzlbg** at 2018-01-17T21:11:50Z
- **mentioned** by **SimonSapin** at 2018-01-17T21:11:50Z
- **subscribed** by **SimonSapin** at 2018-01-17T21:11:50Z
- **mentioned** by **sfackler** at 2018-01-17T21:12:52Z
- **subscribed** by **sfackler** at 2018-01-17T21:12:52Z
- **commented** by **sfackler** at 2018-01-17T21:15:13Z
- **commented** by **SimonSapin** at 2018-01-17T21:26:05Z
- **mentioned** by **Ericson2314** at 2018-01-17T21:26:05Z
- **subscribed** by **Ericson2314** at 2018-01-17T21:26:05Z
- **commented** by **joshlf** at 2018-01-17T21:31:15Z
- **commented** by **gnzlbg** at 2018-01-18T09:10:46Z
- **mentioned** by **Ericson2314** at 2018-01-18T09:10:46Z
- **subscribed** by **Ericson2314** at 2018-01-18T09:10:46Z
- **mentioned** by **SimonSapin** at 2018-01-18T09:10:46Z
- **subscribed** by **SimonSapin** at 2018-01-18T09:10:46Z
- **mentioned** by **joshlf** at 2018-01-18T09:10:46Z
- **subscribed** by **joshlf** at 2018-01-18T09:10:46Z
- **commented** by **hanna-kruppe** at 2018-01-18T10:49:10Z
- **commented** by **rolandsteiner** at 2018-01-18T11:09:54Z
- **commented** by **gnzlbg** at 2018-01-18T13:40:18Z
- **mentioned** by **hanna-kruppe** at 2018-01-18T13:40:18Z
- **subscribed** by **hanna-kruppe** at 2018-01-18T13:40:18Z
- **mentioned** by **Ericson2314** at 2018-01-18T13:40:18Z
- **subscribed** by **Ericson2314** at 2018-01-18T13:40:18Z
- **commented** by **hanna-kruppe** at 2018-01-18T14:06:51Z
- **mentioned** by **gnzlbg** at 2018-01-18T14:06:51Z
- **subscribed** by **gnzlbg** at 2018-01-18T14:06:51Z
- **commented** by **gnzlbg** at 2018-01-18T15:21:14Z
- **commented** by **Ericson2314** at 2018-01-18T16:59:19Z
- **mentioned** by **gnzlbg** at 2018-01-18T16:59:19Z
- **subscribed** by **gnzlbg** at 2018-01-18T16:59:19Z
- **mentioned** by **hanna-kruppe** at 2018-01-18T16:59:19Z
- **subscribed** by **hanna-kruppe** at 2018-01-18T16:59:20Z
- **commented** by **hanna-kruppe** at 2018-01-18T17:08:21Z
- **commented** by **joshlf** at 2018-01-18T17:16:54Z
- **mentioned** by **gnzlbg** at 2018-01-18T17:16:54Z
- **subscribed** by **gnzlbg** at 2018-01-18T17:16:54Z
- **commented** by **raphaelcohn** at 2018-01-19T11:32:13Z
- **mentioned** by **gnzlbg** at 2018-01-19T11:32:13Z
- **subscribed** by **gnzlbg** at 2018-01-19T11:32:13Z
- **commented** by **rpjohnst** at 2018-01-19T17:54:39Z
- **mentioned** by **raphaelcohn** at 2018-01-19T17:54:39Z
- **subscribed** by **raphaelcohn** at 2018-01-19T17:54:39Z
- **commented** by **raphaelcohn** at 2018-01-20T18:05:47Z
- **mentioned** by **rpjohnst** at 2018-01-20T18:05:47Z
- **subscribed** by **rpjohnst** at 2018-01-20T18:05:47Z
- **commented** by **Ericson2314** at 2018-01-21T16:36:21Z
- **commented** by **gnzlbg** at 2018-01-21T17:34:02Z
- **cross-referenced** by **sfleischman105** at 2018-01-30T06:01:52Z
- **cross-referenced** by **sfleischman105** at 2018-01-30T06:05:56Z
- **commented** by **emoon** at 2018-01-31T18:44:23Z
- **commented** by **joshlf** at 2018-01-31T19:10:11Z
- **mentioned** by **emoon** at 2018-01-31T19:10:11Z
- **subscribed** by **emoon** at 2018-01-31T19:10:11Z
- **commented** by **rpjohnst** at 2018-01-31T20:51:03Z
- **commented** by **emoon** at 2018-02-01T10:07:18Z
- **cross-referenced** by **main--** at 2018-02-03T17:28:15Z
- **commented** by **alexreg** at 2018-02-09T18:29:17Z
- **commented** by **Ericson2314** at 2018-02-09T22:59:31Z
- **commented** by **alexreg** at 2018-02-10T00:42:01Z
- **mentioned** by **Ericson2314** at 2018-02-10T00:42:01Z
- **subscribed** by **Ericson2314** at 2018-02-10T00:42:01Z
- **commented** by **cramertj** at 2018-02-10T01:13:16Z
- **cross-referenced** by **fitzgen** at 2018-02-15T23:30:17Z
- **commented** by **brunoczim** at 2018-02-20T18:35:19Z
- **commented** by **SimonSapin** at 2018-02-20T20:43:48Z
- **cross-referenced** by **joshlf** at 2018-02-23T21:16:30Z
- **commented** by **glandium** at 2018-02-25T21:53:47Z
- **commented** by **SimonSapin** at 2018-02-25T22:00:06Z
- **mentioned** by **glandium** at 2018-02-25T22:00:06Z
- **subscribed** by **glandium** at 2018-02-25T22:00:06Z
- **commented** by **glandium** at 2018-02-25T22:10:32Z
- **commented** by **sfackler** at 2018-02-25T22:57:27Z
- **commented** by **joshlf** at 2018-02-25T23:11:53Z
- **commented** by **scottlamb** at 2018-02-26T05:18:31Z
- **commented** by **glandium** at 2018-03-04T11:56:28Z
- **commented** by **sfackler** at 2018-03-04T17:53:56Z
- **commented** by **joshlf** at 2018-03-04T21:12:42Z
- **commented** by **glandium** at 2018-03-04T22:07:11Z
- **commented** by **hanna-kruppe** at 2018-03-04T22:24:01Z
- **commented** by **SimonSapin** at 2018-03-04T22:55:26Z
- **commented** by **glandium** at 2018-03-04T23:30:52Z
- **commented** by **SimonSapin** at 2018-03-05T06:59:37Z
- **commented** by **joshlf** at 2018-03-05T10:05:18Z
- **commented** by **hanna-kruppe** at 2018-03-05T11:53:39Z
- **mentioned** by **SimonSapin** at 2018-03-05T11:53:39Z
- **subscribed** by **SimonSapin** at 2018-03-05T11:53:39Z
- **commented** by **Ericson2314** at 2018-03-05T20:59:10Z
- **commented** by **cramertj** at 2018-03-05T21:06:16Z
- **commented** by **Ericson2314** at 2018-03-05T21:14:36Z
- **mentioned** by **cramertj** at 2018-03-05T21:14:36Z
- **subscribed** by **cramertj** at 2018-03-05T21:14:36Z
- **commented** by **Ericson2314** at 2018-03-05T21:17:13Z
- **commented** by **fitzgen** at 2018-03-05T21:24:35Z
- **mentioned** by **glandium** at 2018-03-05T21:24:35Z
- **subscribed** by **glandium** at 2018-03-05T21:24:35Z
- **commented** by **hanna-kruppe** at 2018-03-05T21:30:22Z
- **mentioned** by **Ericson2314** at 2018-03-05T21:30:22Z
- **subscribed** by **Ericson2314** at 2018-03-05T21:30:22Z
- **mentioned** by **cramertj** at 2018-03-05T21:30:22Z
- **subscribed** by **cramertj** at 2018-03-05T21:30:22Z
- **mentioned** by **SimonSapin** at 2018-03-05T21:30:22Z
- **subscribed** by **SimonSapin** at 2018-03-05T21:30:22Z
- **commented** by **Ericson2314** at 2018-03-05T22:47:19Z
- **commented** by **gnzlbg** at 2018-03-06T11:48:09Z
- **commented** by **SimonSapin** at 2018-03-06T12:50:31Z
- **commented** by **glandium** at 2018-03-28T07:46:27Z
- **commented** by **Ericson2314** at 2018-03-28T18:06:00Z
- **commented** by **SimonSapin** at 2018-03-28T18:24:56Z
- **commented** by **glandium** at 2018-03-29T02:09:36Z
- **commented** by **tomaka** at 2018-03-29T13:19:51Z
- **commented** by **glandium** at 2018-03-29T13:57:15Z
- **commented** by **emilio** at 2018-03-29T15:17:20Z
- **mentioned** by **nox** at 2018-03-29T15:17:20Z
- **subscribed** by **nox** at 2018-03-29T15:17:20Z
- **commented** by **glandium** at 2018-03-29T21:15:30Z
- **commented** by **SimonSapin** at 2018-03-29T23:40:43Z
- **cross-referenced** by **glandium** at 2018-03-30T02:40:42Z
- **referenced** by **glandium** at 2018-04-02T02:16:22Z
- **referenced** by **glandium** at 2018-04-02T02:17:33Z
- **cross-referenced** by **glandium** at 2018-04-02T02:17:59Z
- **referenced** by **glandium** at 2018-04-02T03:29:39Z
- **referenced** by **glandium** at 2018-04-03T00:05:17Z
- **cross-referenced** by **glandium** at 2018-04-03T00:09:27Z
- **referenced** by **glandium** at 2018-04-03T01:32:25Z
- **referenced** by **glandium** at 2018-04-03T03:05:35Z
- **referenced** by **glandium** at 2018-04-03T04:33:59Z
- **referenced** by **glandium** at 2018-04-03T06:21:36Z
- **cross-referenced** by **matthewjasper** at 2018-04-03T22:02:46Z
- **referenced** by **bors** at 2018-04-04T03:48:30Z
- **referenced** by **glandium** at 2018-04-04T04:10:44Z
- **cross-referenced** by **SimonSapin** at 2018-04-04T21:39:17Z
- **commented** by **SimonSapin** at 2018-04-04T21:45:50Z
- **commented** by **alexreg** at 2018-04-04T22:10:35Z
- **commented** by **sfackler** at 2018-04-04T22:13:17Z
- **commented** by **alexreg** at 2018-04-04T22:24:44Z
- **mentioned** by **sfackler** at 2018-04-04T22:24:44Z
- **subscribed** by **sfackler** at 2018-04-04T22:24:44Z
- **commented** by **cramertj** at 2018-04-04T22:37:15Z
- **commented** by **SimonSapin** at 2018-04-04T22:49:35Z
- **commented** by **glandium** at 2018-04-04T23:00:00Z
- **commented** by **glandium** at 2018-04-04T23:03:22Z
- **commented** by **alexreg** at 2018-04-04T23:05:09Z
- **mentioned** by **cramertj** at 2018-04-04T23:05:09Z
- **subscribed** by **cramertj** at 2018-04-04T23:05:09Z
- **mentioned** by **SimonSapin** at 2018-04-04T23:05:09Z
- **subscribed** by **SimonSapin** at 2018-04-04T23:05:09Z
- **mentioned** by **glandium** at 2018-04-04T23:05:09Z
- **subscribed** by **glandium** at 2018-04-04T23:05:09Z
- **commented** by **glandium** at 2018-04-04T23:08:03Z
- **commented** by **alexreg** at 2018-04-04T23:47:18Z
- **mentioned** by **glandium** at 2018-04-04T23:47:18Z
- **subscribed** by **glandium** at 2018-04-04T23:47:18Z
- **commented** by **Amanieu** at 2018-04-04T23:57:27Z
- **commented** by **glandium** at 2018-04-04T23:58:16Z
- **commented** by **alexreg** at 2018-04-05T00:55:12Z
- **mentioned** by **glandium** at 2018-04-05T00:55:12Z
- **subscribed** by **glandium** at 2018-04-05T00:55:12Z
- **commented** by **glandium** at 2018-04-05T00:59:20Z
- **commented** by **alexreg** at 2018-04-05T01:30:51Z
- **mentioned** by **glandium** at 2018-04-05T01:30:51Z
- **subscribed** by **glandium** at 2018-04-05T01:30:51Z
- **commented** by **gnzlbg** at 2018-04-05T06:39:22Z
- **mentioned** by **Amanieu** at 2018-04-05T06:39:22Z
- **subscribed** by **Amanieu** at 2018-04-05T06:39:22Z
- **referenced** by **Robbepop** at 2018-04-08T11:02:22Z
- **commented** by **glandium** at 2018-05-03T21:53:45Z
- **commented** by **glandium** at 2018-05-03T23:41:55Z
- **commented** by **gnzlbg** at 2018-05-04T09:44:08Z
- **mentioned** by **glandium** at 2018-05-04T09:44:08Z
- **subscribed** by **glandium** at 2018-05-04T09:44:09Z
- **commented** by **glandium** at 2018-05-04T09:48:56Z
- **commented** by **retep998** at 2018-05-07T20:27:05Z
- **commented** by **ruuda** at 2018-05-07T21:32:42Z
- **commented** by **gnzlbg** at 2018-05-07T21:35:19Z
- **commented** by **ruuda** at 2018-05-07T22:01:19Z
- **commented** by **glandium** at 2018-05-07T22:20:55Z
- **commented** by **gnzlbg** at 2018-05-07T22:32:33Z
- **commented** by **glandium** at 2018-05-10T10:08:20Z
- **labeled** by **Centril** at 2018-05-24T23:11:46Z
- **labeled** by **Centril** at 2018-05-24T23:11:46Z
- **unlabeled** by **Centril** at 2018-05-24T23:11:46Z
- **commented** by **Amanieu** at 2018-05-29T11:12:18Z
- **commented** by **gnzlbg** at 2018-05-29T11:34:26Z
- **commented** by **Amanieu** at 2018-05-29T20:15:03Z
- **commented** by **joshlf** at 2018-05-29T20:21:23Z
- **commented** by **Amanieu** at 2018-05-29T20:37:49Z
- **commented** by **alexreg** at 2018-05-29T20:39:12Z
- **mentioned** by **Ericson2314** at 2018-05-29T20:39:12Z
- **subscribed** by **Ericson2314** at 2018-05-29T20:39:12Z
- **commented** by **joshlf** at 2018-05-29T20:49:45Z
- **commented** by **Amanieu** at 2018-05-29T22:07:53Z
- **mentioned** by **joshlf** at 2018-05-29T22:07:53Z
- **subscribed** by **joshlf** at 2018-05-29T22:07:53Z
- **commented** by **joshlf** at 2018-05-29T22:19:11Z
- **commented** by **eupp** at 2018-06-12T19:43:24Z
- **commented** by **cramertj** at 2018-06-12T19:56:44Z
- **mentioned** by **eupp** at 2018-06-12T19:56:44Z
- **subscribed** by **eupp** at 2018-06-12T19:56:44Z
- **commented** by **eupp** at 2018-06-12T20:07:49Z
- **mentioned** by **cramertj** at 2018-06-12T20:07:49Z
- **subscribed** by **cramertj** at 2018-06-12T20:07:49Z
- **commented** by **remexre** at 2018-06-12T20:09:50Z
- **mentioned** by **cramertj** at 2018-06-12T20:09:50Z
- **subscribed** by **cramertj** at 2018-06-12T20:09:50Z
- **commented** by **eupp** at 2018-06-12T20:39:11Z
- **mentioned** by **remexre** at 2018-06-12T20:39:11Z
- **subscribed** by **remexre** at 2018-06-12T20:39:11Z
- **commented** by **SimonSapin** at 2018-06-12T20:59:38Z
- **commented** by **SimonSapin** at 2018-06-12T21:03:07Z
- **commented** by **cramertj** at 2018-06-12T21:13:02Z
- **mentioned** by **SimonSapin** at 2018-06-12T21:13:02Z
- **subscribed** by **SimonSapin** at 2018-06-12T21:13:02Z
- **commented** by **eupp** at 2018-06-13T07:21:12Z
- **mentioned** by **cramertj** at 2018-06-13T07:21:12Z
- **subscribed** by **cramertj** at 2018-06-13T07:21:12Z
- **commented** by **sfackler** at 2018-06-13T16:00:20Z
- **commented** by **cramertj** at 2018-06-13T16:46:48Z
- **mentioned** by **sfackler** at 2018-06-13T16:46:48Z
- **subscribed** by **sfackler** at 2018-06-13T16:46:48Z
- **commented** by **the8472** at 2018-07-04T21:07:59Z
- **commented** by **joshlf** at 2018-07-04T21:51:41Z
- **commented** by **glandium** at 2018-07-04T21:57:42Z
- **commented** by **the8472** at 2018-07-04T22:03:24Z
- **commented** by **the8472** at 2018-07-05T18:34:27Z
- **commented** by **gnzlbg** at 2018-07-06T12:01:33Z
- **commented** by **the8472** at 2018-07-06T18:58:20Z
- **cross-referenced** by **FenrirWolf** at 2018-08-06T19:27:34Z
- **commented** by **shanemikel** at 2018-09-06T00:53:14Z
- **commented** by **gnzlbg** at 2018-09-06T09:27:24Z
- **commented** by **Amanieu** at 2018-10-25T17:33:39Z
- **commented** by **gnzlbg** at 2018-10-25T17:44:59Z
- **commented** by **Amanieu** at 2018-10-25T17:46:29Z
- **mentioned** by **gnzlbg** at 2018-10-25T17:46:29Z
- **subscribed** by **gnzlbg** at 2018-10-25T17:46:29Z
- **commented** by **SimonSapin** at 2018-10-25T22:06:52Z
- **mentioned** by **Amanieu** at 2018-10-25T22:06:52Z
- **subscribed** by **Amanieu** at 2018-10-25T22:06:52Z
- **cross-referenced** by **Amanieu** at 2018-10-25T22:54:58Z
- **referenced** by **bors** at 2018-11-07T02:28:20Z
- **referenced** by **bors** at 2018-11-08T06:52:39Z
- **cross-referenced** by **gnzlbg** at 2018-12-02T17:19:57Z
- **commented** by **TimDiekmann** at 2019-02-24T21:15:40Z
- **commented** by **SimonSapin** at 2019-02-24T23:20:27Z
- **commented** by **shanemikel** at 2019-02-25T00:09:07Z
- **mentioned** by **gnzlbg** at 2019-02-25T00:09:07Z
- **subscribed** by **gnzlbg** at 2019-02-25T00:09:07Z
- **commented** by **withoutboats** at 2019-02-25T02:12:22Z
- **commented** by **TimDiekmann** at 2019-02-25T09:05:07Z
- **commented** by **TimDiekmann** at 2019-02-25T13:24:39Z
- **commented** by **gnzlbg** at 2019-02-25T16:56:49Z
- **commented** by **TimDiekmann** at 2019-02-25T17:07:38Z
- **commented** by **gnzlbg** at 2019-02-25T17:15:39Z
- **commented** by **TimDiekmann** at 2019-02-25T17:28:56Z
- **mentioned** by **pnkfelix** at 2019-02-25T17:28:56Z
- **subscribed** by **pnkfelix** at 2019-02-25T17:28:56Z
- **commented** by **gnzlbg** at 2019-02-25T18:10:01Z
- **commented** by **TimDiekmann** at 2019-02-25T18:36:58Z
- **commented** by **SimonSapin** at 2019-03-10T19:40:42Z
- **subscribed** by **alexcrichton** at 2019-03-10T19:40:42Z
- **subscribed** by **Amanieu** at 2019-03-10T19:40:42Z
- **subscribed** by **SimonSapin** at 2019-03-10T19:40:42Z
- **subscribed** by **BurntSushi** at 2019-03-10T19:40:42Z
- **subscribed** by **sfackler** at 2019-03-10T19:40:42Z
- **subscribed** by **dtolnay** at 2019-03-10T19:40:42Z
- **subscribed** by **Kimundi** at 2019-03-10T19:40:42Z
- **subscribed** by **withoutboats** at 2019-03-10T19:40:42Z
- **commented** by **joshlf** at 2019-03-11T07:15:10Z
- **commented** by **gnzlbg** at 2019-03-11T08:04:22Z
- **commented** by **joshlf** at 2019-03-11T15:44:08Z
- **commented** by **burdges** at 2019-03-11T16:14:53Z
- **commented** by **joshlf** at 2019-03-11T16:17:10Z
- **commented** by **gnzlbg** at 2019-03-11T16:17:24Z
- **commented** by **joshlf** at 2019-03-11T16:21:26Z
- **cross-referenced** by **SimonSapin** at 2019-03-21T21:41:27Z
- **commented** by **brson** at 2019-04-04T21:40:44Z
- **commented** by **TimDiekmann** at 2019-04-04T21:54:49Z
- **commented** by **gnzlbg** at 2019-04-05T14:07:41Z
- **commented** by **raphaelcohn** at 2019-04-12T07:14:34Z
- **mentioned** by **brson** at 2019-04-12T07:14:34Z
- **subscribed** by **brson** at 2019-04-12T07:14:34Z
- **commented** by **gnzlbg** at 2019-04-12T08:47:24Z
- **mentioned** by **raphaelcohn** at 2019-04-12T08:47:24Z
- **subscribed** by **raphaelcohn** at 2019-04-12T08:47:24Z
- **commented** by **gnzlbg** at 2019-04-12T09:36:51Z
- **cross-referenced** by **TimDiekmann** at 2019-05-04T12:37:15Z
- **commented** by **TimDiekmann** at 2019-05-04T12:40:01Z
- **commented** by **alexcrichton** at 2019-05-06T15:16:48Z
- **closed** by **alexcrichton** at 2019-05-06T15:16:49Z
- **mentioned** by **TimDiekmann** at 2019-05-06T15:16:49Z
- **subscribed** by **TimDiekmann** at 2019-05-06T15:16:49Z
- **commented** by **SimonSapin** at 2019-05-06T15:50:46Z
- **commented** by **jethrogb** at 2019-05-06T17:30:56Z
- **commented** by **Gankra** at 2019-05-06T17:47:57Z
- **reopened** by **Gankra** at 2019-05-06T17:47:58Z
- **cross-referenced** by **godcodehunter** at 2019-11-16T04:03:34Z
- **cross-referenced** by **mbrubeck** at 2019-11-19T13:54:36Z
- **cross-referenced** by **TimDiekmann** at 2020-01-17T12:05:43Z
- **cross-referenced** by **asutherland** at 2020-02-11T21:29:13Z
- **cross-referenced** by **rubdos** at 2020-02-14T10:14:46Z
- **cross-referenced** by **gereeter** at 2020-02-25T17:10:12Z
- **referenced** by **makubacki** at 2020-04-21T06:55:46Z
- **referenced** by **uefibot** at 2020-04-22T00:06:10Z
- **referenced** by **makubacki** at 2020-04-23T20:40:02Z
- **referenced** by **makubacki** at 2020-04-27T21:40:43Z
- **referenced** by **makubacki** at 2020-04-29T09:37:45Z
- **referenced** by **corthon** at 2020-06-16T07:27:16Z
- **referenced** by **corthon** at 2020-06-17T01:58:28Z
- **cross-referenced** by **o0Ignition0o** at 2020-06-29T13:18:18Z
- **cross-referenced** by **spastorino** at 2020-07-09T14:21:28Z
- **labeled** by **KodrAus** at 2020-07-29T22:02:01Z
- **cross-referenced** by **TimDiekmann** at 2020-08-05T10:03:58Z
- **referenced** by **jacob-hughes** at 2020-09-23T15:01:14Z
- **cross-referenced** by **jacob-hughes** at 2020-09-23T15:04:29Z
- **referenced** by **bors[bot]** at 2020-09-23T16:24:55Z
- **cross-referenced** by **timothee-haudebourg** at 2020-12-22T21:11:00Z
- **cross-referenced** by **yoshuawuyts** at 2021-01-15T13:21:55Z
- **cross-referenced** by **ojeda** at 2021-03-08T21:26:43Z
- **cross-referenced** by **ryoqun** at 2021-03-30T04:17:36Z
- **cross-referenced** by **Ericson2314** at 2021-07-07T15:13:57Z
- **cross-referenced** by **kevinaboos** at 2021-07-27T18:24:07Z
- **cross-referenced** by **CertainLach** at 2021-08-21T07:39:28Z
- **cross-referenced** by **kornelski** at 2021-10-04T14:50:58Z
- **cross-referenced** by **Veeupup** at 2021-10-13T14:01:03Z
- **cross-referenced** by **sdd** at 2021-10-25T07:53:12Z
- **cross-referenced** by **dead-claudia** at 2021-11-06T01:48:33Z
- **cross-referenced** by **nbgl** at 2021-11-09T21:03:33Z
- **cross-referenced** by **Tamschi** at 2021-11-19T18:48:12Z
- **cross-referenced** by **myrrlyn** at 2021-11-20T19:10:40Z
- **commented** by **adsnaider** at 2021-11-23T03:27:56Z
- **referenced** by **matthiaskrgr** at 2021-11-23T18:28:09Z
- **cross-referenced** by **hexiaowen** at 2021-12-21T03:25:50Z
- **labeled** by **joshtriplett** at 2022-01-19T18:50:30Z
- **cross-referenced** by **VictorKoenders** at 2022-02-01T14:15:18Z
- **cross-referenced** by **mbrubeck** at 2022-04-18T05:32:37Z
- **labeled** by **RalfJung** at 2022-07-04T03:27:50Z
- **commented** by **RalfJung** at 2022-07-04T03:30:17Z
- **cross-referenced** by **yanchith** at 2022-07-16T17:34:03Z
- **cross-referenced** by **aabizri** at 2022-08-21T15:30:13Z
- **cross-referenced** by **scottmcm** at 2022-09-15T05:25:10Z
- **commented** by **scottmcm** at 2022-09-15T05:32:58Z
- **commented** by **thomcc** at 2022-09-15T05:43:48Z
- **commented** by **RalfJung** at 2022-09-15T05:58:43Z
- **commented** by **CraftSpider** at 2022-09-16T00:38:57Z
- **cross-referenced** by **nicholasbishop** at 2022-11-24T21:20:32Z
- **commented** by **programmerjake** at 2023-01-18T18:00:33Z
- **subscribed** by **wilzbach** at 2023-02-13T21:41:53Z
- **cross-referenced** by **rafaelcaricio** at 2023-03-02T11:26:49Z
- **subscribed** by **epbuennig** at 2023-03-12T18:41:42Z
- **commented** by **ghost** at 2023-03-24T00:21:43Z
- **cross-referenced** by **coder137** at 2023-04-05T03:25:05Z
- **cross-referenced** by **tleibert** at 2023-04-18T03:27:08Z
- **cross-referenced** by **TeleportAura** at 2023-04-29T21:24:22Z
- **cross-referenced** by **chuigda** at 2023-05-06T05:02:41Z
- **unlabeled** by **Amanieu** at 2023-05-15T15:23:00Z
- **unlabeled** by **Amanieu** at 2023-05-15T15:23:10Z
- **referenced** by **jacob-hughes** at 2023-05-17T12:32:48Z
- **commented** by **Amanieu** at 2023-05-19T16:47:09Z
- **commented** by **Ericson2314** at 2023-05-19T16:57:17Z
- **commented** by **Amanieu** at 2023-05-19T21:34:18Z
- **commented** by **Jules-Bertholet** at 2023-05-19T21:55:07Z
- **commented** by **thomcc** at 2023-05-19T21:59:48Z
- **commented** by **Amanieu** at 2023-05-19T22:04:49Z
- **commented** by **thomcc** at 2023-05-19T22:18:27Z
- **commented** by **withoutboats** at 2023-05-19T22:28:34Z
- **commented** by **Lokathor** at 2023-05-19T22:31:48Z
- **commented** by **RalfJung** at 2023-05-20T07:05:05Z
- **subscribed** by **BugenZhao** at 2023-05-22T03:44:57Z
- **subscribed** by **Wodann** at 2023-05-25T04:52:37Z
- **commented** by **yanchith** at 2023-06-09T10:51:56Z
- **mentioned** by **thomcc** at 2023-06-09T10:51:57Z
- **subscribed** by **thomcc** at 2023-06-09T10:51:57Z
- **commented** by **thomcc** at 2023-06-10T14:16:41Z
- **commented** by **thomcc** at 2023-06-10T15:33:34Z
- **commented** by **thomcc** at 2023-06-10T21:47:46Z
- **mentioned** by **Amanieu** at 2023-06-10T21:47:46Z
- **subscribed** by **Amanieu** at 2023-06-10T21:47:46Z
- **commented** by **Amanieu** at 2023-06-11T00:17:51Z
- **commented** by **thomcc** at 2023-06-11T01:25:14Z
- **commented** by **nbdd0121** at 2023-06-11T11:43:15Z
- **commented** by **thomcc** at 2023-06-12T05:24:05Z
- **unsubscribed** by **yanchith** at 2023-06-12T05:38:54Z
- **referenced** by **bors** at 2023-06-12T20:43:52Z
- **referenced** by **bors** at 2023-06-13T03:23:16Z
- **commented** by **RalfJung** at 2023-06-16T13:28:26Z
- **commented** by **victoryaskevich** at 2023-06-16T14:01:55Z
- **cross-referenced** by **udoprog** at 2023-06-23T01:16:07Z
- **commented** by **thomcc** at 2023-06-23T06:37:35Z
- **cross-referenced** by **thomcc** at 2023-06-23T07:51:59Z
- **commented** by **RalfJung** at 2023-06-25T07:11:19Z
- **commented** by **thomcc** at 2023-06-25T07:18:16Z
- **commented** by **RalfJung** at 2023-06-25T13:18:13Z
- **commented** by **thomcc** at 2023-06-26T10:15:14Z
- **commented** by **RalfJung** at 2023-06-26T11:46:53Z
- **commented** by **the8472** at 2023-06-26T14:41:50Z
- **mentioned** by **thomcc** at 2023-06-26T14:41:50Z
- **subscribed** by **thomcc** at 2023-06-26T14:41:50Z
- **commented** by **Amanieu** at 2023-06-28T01:21:22Z
- **commented** by **lilith** at 2023-06-28T02:39:02Z
- **commented** by **Amanieu** at 2023-06-28T02:40:15Z
- **commented** by **lilith** at 2023-06-28T02:42:39Z
- **commented** by **thomcc** at 2023-06-28T07:43:33Z
- **commented** by **RalfJung** at 2023-06-28T11:05:03Z
- **commented** by **Jules-Bertholet** at 2023-06-28T12:42:59Z
- **commented** by **thomcc** at 2023-06-28T14:25:36Z
- **commented** by **RalfJung** at 2023-06-28T15:22:01Z
- **mentioned** by **thomcc** at 2023-06-28T15:22:02Z
- **subscribed** by **thomcc** at 2023-06-28T15:22:02Z
- **commented** by **nbdd0121** at 2023-06-28T16:12:24Z
- **commented** by **RalfJung** at 2023-06-28T16:18:36Z
- **commented** by **Jules-Bertholet** at 2023-06-28T16:32:35Z
- **commented** by **thomcc** at 2023-06-28T21:02:09Z
- **commented** by **RalfJung** at 2023-06-28T21:06:29Z
- **commented** by **thomcc** at 2023-06-28T21:10:14Z
- **commented** by **RalfJung** at 2023-06-29T06:04:12Z
- **commented** by **thomcc** at 2023-06-29T07:36:42Z
- **commented** by **thomcc** at 2023-08-07T03:09:18Z
- **unsubscribed** by **kzdnk** at 2023-09-06T23:33:06Z
- **subscribed** by **ErrorNoInternet** at 2023-09-24T18:47:24Z
- **cross-referenced** by **Coekjan** at 2023-10-29T05:52:16Z
- **cross-referenced** by **ruihe774** at 2023-11-04T18:05:14Z
- **cross-referenced** by **dylanplecki** at 2024-01-19T20:37:01Z
- **commented** by **wallgeek** at 2024-01-23T18:47:52Z
- **commented** by **udoprog** at 2024-01-23T19:26:39Z
- **mentioned** by **wallgeek** at 2024-01-23T19:26:39Z
- **subscribed** by **wallgeek** at 2024-01-23T19:26:39Z
- **commented** by **pravic** at 2024-01-24T07:14:35Z
- **commented** by **SimonSapin** at 2024-01-24T21:16:33Z
- **unsubscribed** by **hcsch** at 2024-02-11T06:37:15Z
- **subscribed** by **hcsch** at 2024-02-11T06:37:17Z
- **cross-referenced** by **Dylan-DPC** at 2024-03-04T12:50:28Z
- **subscribed** by **JoHaHu** at 2024-03-05T02:05:16Z
- **subscribed** by **N9199** at 2024-03-07T00:19:55Z
- **cross-referenced** by **oli-obk** at 2024-06-04T08:19:01Z
- **cross-referenced** by **AljoschaMeyer** at 2024-07-06T07:13:52Z
- **referenced** by **jieyouxu** at 2024-07-22T08:44:04Z
- **referenced** by **rust-timer** at 2024-07-22T11:24:37Z
- **cross-referenced** by **jelmer** at 2024-08-16T09:26:51Z
- **subscribed** by **raftario** at 2024-09-17T18:59:17Z
- **unsubscribed** by **theli-ua** at 2024-09-20T17:48:42Z
- **subscribed** by **theli-ua** at 2024-09-20T17:48:45Z
- **subscribed** by **bjoernager** at 2024-09-22T20:24:06Z
- **subscribed** by **FeldrinH** at 2024-09-25T12:58:51Z
- **subscribed** by **RohitPatil555** at 2024-09-26T16:31:36Z
- **subscribed** by **Mathspy** at 2024-10-15T19:05:00Z
- **subscribed** by **akhilman** at 2024-10-18T22:50:53Z
- **subscribed** by **skewballfox** at 2024-10-20T13:17:49Z
- **subscribed** by **Conaclos** at 2024-10-25T15:09:07Z
- **subscribed** by **raviqqe** at 2024-10-26T15:14:55Z
- **commented** by **vmolsa** at 2024-10-31T04:20:51Z
- **subscribed** by **dani-garcia** at 2024-11-04T14:42:42Z
- **subscribed** by **K0bin** at 2024-11-12T10:18:20Z
- **unsubscribed** by **mogud** at 2024-11-12T16:03:31Z
- **subscribed** by **meowette** at 2024-11-19T14:16:54Z
- **subscribed** by **liff** at 2024-11-21T14:19:18Z
- **cross-referenced** by **RalfJung** at 2024-11-30T10:58:21Z
- **unsubscribed** by **SmiteWindows** at 2024-11-30T12:50:26Z
- **unsubscribed** by **wrongnull** at 2024-12-01T12:55:25Z
- **unsubscribed** by **wrongnull** at 2024-12-01T12:55:27Z
- **subscribed** by **SapryWenInera** at 2024-12-02T00:13:44Z
- **unsubscribed** by **SapryWenInera** at 2024-12-02T00:14:29Z
- **commented** by **Sewer56** at 2024-12-05T22:28:47Z
- **subscribed** by **rami3l** at 2024-12-06T00:34:44Z
- **subscribed** by **andreivasiliu** at 2024-12-06T21:27:05Z
- **subscribed** by **orzogc** at 2024-12-07T15:31:25Z
- **subscribed** by **unknown** at 2024-12-12T01:39:56Z
- **subscribed** by **ryze312** at 2024-12-12T15:07:35Z
- **subscribed** by **LennyLizowzskiy** at 2024-12-17T12:56:24Z
- **subscribed** by **unknown** at 2024-12-26T15:58:49Z
- **unsubscribed** by **eliaskoromilas** at 2024-12-27T20:04:04Z
- **subscribed** by **kei519** at 2025-01-03T10:30:06Z
- **subscribed** by **kei519** at 2025-01-03T10:30:12Z
- **subscribed** by **White-Green** at 2025-01-05T23:51:36Z
- **subscribed** by **HackerFoo** at 2025-01-10T19:11:46Z
- **subscribed** by **ullebe1** at 2025-01-13T05:52:24Z
- **subscribed** by **Lokathor** at 2025-01-13T08:13:41Z
- **subscribed** by **Qix-** at 2025-01-15T13:20:23Z
- **subscribed** by **rpodgorny** at 2025-01-17T15:04:31Z
- **subscribed** by **interruptinuse** at 2025-01-24T13:14:04Z
- **subscribed** by **interruptinuse** at 2025-01-24T13:14:10Z
- **subscribed** by **zslayton** at 2025-01-29T19:57:06Z
- **commented** by **v-thakkar** at 2025-02-10T06:32:43Z
- **mentioned** by **Sewer56** at 2025-02-10T06:32:44Z
- **subscribed** by **Sewer56** at 2025-02-10T06:32:44Z
- **subscribed** by **zzhaolei** at 2025-02-10T06:40:00Z
- **subscribed** by **LambdaAlpha** at 2025-02-11T03:10:29Z
- **unsubscribed** by **liquidev** at 2025-02-12T16:40:06Z
- **subscribed** by **Determinant** at 2025-02-13T06:36:51Z
- **subscribed** by **kaphula** at 2025-02-15T05:12:33Z
- **subscribed** by **zxfsee** at 2025-02-16T15:09:19Z
- **subscribed** by **win-t** at 2025-02-24T03:30:41Z
- **subscribed** by **win-t** at 2025-02-27T10:48:37Z
- **subscribed** by **robUx4** at 2025-03-06T07:01:47Z
- **cross-referenced** by **ljedrz** at 2025-03-10T12:12:32Z
- **commented** by **abgros** at 2025-03-28T06:09:59Z
- **commented** by **abgros** at 2025-03-28T06:39:04Z
- **commented** by **programmerjake** at 2025-03-28T06:50:56Z
- **commented** by **abgros** at 2025-03-28T07:11:00Z
- **unsubscribed** by **jelmer** at 2025-03-28T11:28:29Z
- **commented** by **Amanieu** at 2025-03-28T11:31:00Z
- **commented** by **RalfJung** at 2025-03-28T12:56:25Z
- **commented** by **Jules-Bertholet** at 2025-03-28T13:32:16Z
- **unsubscribed** by **ciehanski** at 2025-03-28T14:12:21Z
- **unsubscribed** by **whispersofthedawn** at 2025-03-29T02:24:28Z
- **subscribed** by **DenialAdams** at 2025-04-01T23:52:59Z
- **commented** by **Keith-Cancel** at 2025-04-07T07:00:26Z
- **subscribed** by **danald** at 2025-04-07T17:17:21Z
- **commented** by **petertodd** at 2025-04-08T11:02:09Z
- **mentioned** by **abgros** at 2025-04-08T11:02:10Z
- **subscribed** by **abgros** at 2025-04-08T11:02:11Z
- **subscribed** by **mstarodub** at 2025-04-08T12:13:31Z
- **cross-referenced** by **ctz** at 2025-04-14T16:43:17Z
- **subscribed** by **martabal** at 2025-04-19T07:45:22Z
- **subscribed** by **Zk2u** at 2025-05-01T08:48:15Z
- **subscribed** by **js-choi** at 2025-05-07T04:33:49Z
- **subscribed** by **mvelbaum** at 2025-05-07T07:45:09Z
- **subscribed** by **estaban** at 2025-05-09T19:38:51Z
- **subscribed** by **sullyj3** at 2025-05-13T08:32:09Z
- **subscribed** by **bconnorwhite** at 2025-05-13T17:53:19Z
- **subscribed** by **alecmocatta** at 2025-05-16T12:06:41Z
- **subscribed** by **ashvardanian** at 2025-05-19T05:56:24Z
- **subscribed** by **Weypare** at 2025-05-20T12:26:12Z
- **subscribed** by **ssmike** at 2025-05-27T05:16:37Z
- **subscribed** by **Eveeifyeve** at 2025-06-06T19:40:49Z
- **commented** by **mikeyhew** at 2025-06-11T21:07:33Z
- **commented** by **Amanieu** at 2025-06-11T21:56:08Z
- **commented** by **programmerjake** at 2025-06-11T22:52:11Z
- **commented** by **mikeyhew** at 2025-06-11T23:22:02Z
- **commented** by **FeldrinH** at 2025-06-11T23:43:43Z
- **cross-referenced** by **programmerjake** at 2025-06-12T03:40:12Z
- **commented** by **petertodd** at 2025-06-17T10:13:11Z
- **subscribed** by **joshuaseaton** at 2025-07-10T19:23:45Z
- **commented** by **jpochyla** at 2025-07-17T18:54:14Z
- **subscribed** by **gaesa** at 2025-07-30T06:17:49Z
- **subscribed** by **CptPotato** at 2025-08-06T13:49:37Z
- **subscribed** by **YtvwlD** at 2025-08-07T09:36:29Z
- **subscribed** by **lucasdietrich** at 2025-08-12T19:14:18Z
- **subscribed** by **K0bin** at 2025-08-21T23:01:18Z
- **subscribed** by **unknown** at 2025-08-24T02:01:55Z
- **subscribed** by **YouSafe** at 2025-08-24T08:31:28Z
- **subscribed** by **reneleonhardt** at 2025-08-26T15:25:24Z
- **subscribed** by **CathalMullan** at 2025-08-28T16:19:03Z
- **subscribed** by **andriilahuta** at 2025-08-30T02:35:06Z
- **subscribed** by **smirzaei** at 2025-09-02T17:36:42Z
- **cross-referenced** by **ultimaweapon** at 2025-09-06T12:03:55Z
- **referenced** by **phip1611** at 2025-09-13T08:28:25Z
- **cross-referenced** by **phip1611** at 2025-09-13T08:28:37Z
- **referenced** by **phip1611** at 2025-09-13T08:41:57Z
- **referenced** by **phip1611** at 2025-09-13T08:47:50Z
- **referenced** by **phip1611** at 2025-09-13T08:50:24Z
- **referenced** by **phip1611** at 2025-09-16T05:31:57Z
- **subscribed** by **Dzuchun** at 2025-09-18T00:38:23Z
- **subscribed** by **regexident** at 2025-09-19T20:51:23Z
- **subscribed** by **bluurryy** at 2025-09-20T23:49:19Z
- **cross-referenced** by **bluurryy** at 2025-09-21T00:21:44Z
- **subscribed** by **boozook** at 2025-09-21T08:25:52Z
- **cross-referenced** by **cosmicexplorer** at 2025-10-01T10:57:17Z
- **cross-referenced** by **Javagedes** at 2025-10-01T23:22:04Z
- **subscribed** by **HRKings** at 2025-10-03T03:57:30Z
- **subscribed** by **dbremner** at 2025-10-04T19:28:03Z
- **subscribed** by **Uroc327** at 2025-10-22T17:48:11Z
- **subscribed** by **kerty0** at 2025-11-03T08:56:25Z
- **subscribed** by **titaniumtraveler** at 2025-11-10T11:27:52Z
- **referenced** by **seijikun** at 2025-11-12T10:53:31Z
- **subscribed** by **nathany** at 2025-11-12T21:33:44Z
- **referenced** by **SteveLauC** at 2025-11-24T02:47:15Z
- **referenced** by **SteveLauC** at 2025-11-24T03:10:31Z
- **cross-referenced** by **shua** at 2025-11-25T20:27:31Z
- **referenced** by **SteveLauC** at 2025-11-27T02:12:50Z
- **referenced** by **SteveLauC** at 2025-12-05T07:32:59Z
- **subscribed** by **IamPyu** at 2025-12-05T18:07:21Z
- **referenced** by **matthiaskrgr** at 2025-12-06T08:57:59Z
- **referenced** by **jhpratt** at 2025-12-06T09:13:43Z
- **subscribed** by **emilycares** at 2025-12-06T14:22:00Z
- **subscribed** by **emilycares** at 2025-12-06T14:22:35Z
- **referenced** by **rust-timer** at 2025-12-06T15:38:43Z
- **cross-referenced** by **stefanschmidt77** at 2025-12-07T09:57:48Z
- **subscribed** by **hypersimplex** at 2025-12-11T01:48:08Z
- **referenced** by **github-actions[bot]** at 2025-12-13T13:33:33Z
- **referenced** by **makai410** at 2025-12-16T14:00:44Z
- **referenced** by **makai410** at 2025-12-16T14:15:26Z
- **subscribed** by **amy-keibler** at 2025-12-17T18:37:50Z
- **commented** by **cosmicexplorer** at 2026-01-02T18:31:05Z
- **subscribed** by **d-e-s-o** at 2026-01-08T04:49:42Z
- **subscribed** by **nomyfan** at 2026-01-13T04:29:14Z
