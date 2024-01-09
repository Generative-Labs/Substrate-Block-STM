// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

// Modifications and additional contributions by GenerativeLabs.
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic;

// Parallel algorithms often guarantee a sequential use of certain
// data structures, or parts of the data-structures (like elements of
// a vector).  The rust compiler can not prove the safety of even
// slightly complex parallel algorithms.

/// ExplicitSyncWrapper is meant to be used in parallel algorithms
/// where we can prove that there will be no concurrent access to the
/// underlying object (or its elements).  Use with caution - only when
/// the safety can be proven.
pub struct ExplicitSyncWrapper<T> {
    value: UnsafeCell<T>,
}

pub struct Guard<'a, T> {
    lock: &'a ExplicitSyncWrapper<T>,
}

impl<T> ExplicitSyncWrapper<T> {
    pub const fn new(value: T) -> Self {
        Self { value: UnsafeCell::new(value) }
    }

    pub fn acquire(&self) -> Guard<T> {
        atomic::fence(atomic::Ordering::Acquire);
        Guard { lock: self }
    }

    pub(crate) fn unlock(&self) {
        atomic::fence(atomic::Ordering::Release);
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }

    pub fn dereference(&self) -> &T {
        unsafe { &*self.value.get() }
    }

    pub fn dereference_mut<'a>(&self) -> &'a mut T {
        unsafe { &mut *self.value.get() }
    }
}

impl<T> Guard<'_, T> {
    pub fn dereference(&self) -> &T {
        self.lock.dereference()
    }

    pub fn dereference_mut(&mut self) -> &mut T {
        self.lock.dereference_mut()
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.lock.dereference()
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.lock.dereference_mut()
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

unsafe impl<T> Sync for ExplicitSyncWrapper<T> {}

use std::sync::Mutex as StdMutex;
pub use std::sync::MutexGuard;

/// A simple wrapper around the lock() function of a std::sync::Mutex
/// The only difference is that you don't need to call unwrap() on it.
#[derive(Debug)]
pub struct Mutex<T>(StdMutex<T>);

impl<T> Mutex<T> {
    /// creates mutex
    pub fn new(t: T) -> Self {
        Self(StdMutex::new(t))
    }

    /// lock the mutex
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.0.lock().expect("Cannot currently handle a poisoned lock")
    }

    // consume the mutex
    pub fn into_inner(self) -> T {
        self.0.into_inner().expect("Cannot currently handle a poisoned lock")
    }
}

impl<T> Default for Mutex<Option<T>> {
    fn default() -> Self {
        Self::new(None)
    }
}