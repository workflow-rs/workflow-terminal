use wasm_bindgen::{JsCast, closure::Closure, convert::FromWasmAbi};
use std::sync::Arc;
use crate::Result;

#[derive(Debug)]
pub struct Listener<T>{
    pub closure:Arc<Closure<dyn FnMut(T)->Result<()>>>
}

impl<T> Clone for Listener<T>{
    fn clone(&self) -> Self {
        Self { closure: self.closure.clone() }
     }
}

impl<T> Listener<T>
where T: Sized + FromWasmAbi + 'static
{
    pub fn new<F>(t:F)->Listener<T> where F: FnMut(T) ->Result<()> + 'static{
        Listener{
            closure: Arc::new(Closure::new(t))
        }
    }
    pub fn into_js<J>(&self) -> &J where J: JsCast{
        (*self.closure).as_ref().unchecked_ref()
    }
}
