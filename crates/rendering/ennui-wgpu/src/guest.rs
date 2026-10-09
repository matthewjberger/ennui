use crate::shading::Shading;
use ennui::prelude::Resources;
use ennui::resources::hold;
use naga_oil::compose::Composer;
use std::any::Any;
use std::sync::Arc;

pub struct Bench<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub surface: wgpu::TextureFormat,
    pub size: [u32; 2],
    pub shading: &'a Shading,
    pub(crate) composer: &'a mut Composer,
}

pub struct Canvas<'a> {
    pub shading: &'a Shading,
}

pub(crate) type Values = dyn Any + Send + Sync;

pub(crate) type Ready = Box<dyn Fn(&mut Values, &Bench, &Resources) + Send + Sync>;
pub(crate) type Draw = Box<dyn Fn(&Values, &mut wgpu::RenderPass<'_>, &Canvas) + Send + Sync>;

pub struct Fitting<T> {
    held: T,
    ready: Option<fn(&mut T, &Bench, &Resources)>,
    draw: Option<fn(&T, &mut wgpu::RenderPass<'_>, &Canvas)>,
}

impl<T> Fitting<T> {
    pub fn of(held: T) -> Self {
        Self {
            held,
            ready: None,
            draw: None,
        }
    }

    pub fn readying(mut self, ready: fn(&mut T, &Bench, &Resources)) -> Self {
        self.ready = Some(ready);
        self
    }

    pub fn drawing(mut self, draw: fn(&T, &mut wgpu::RenderPass<'_>, &Canvas)) -> Self {
        self.draw = Some(draw);
        self
    }
}

pub(crate) struct Planted {
    pub(crate) name: &'static str,
    pub(crate) held: Box<Values>,
    pub(crate) ready: Option<Ready>,
    pub(crate) draw: Option<Draw>,
}

fn typed<T: Send + Sync + 'static>(held: &Values) -> &T {
    held.downcast_ref::<T>()
        .expect("a guest callback is given the state its fitting held")
}

fn typed_mut<T: Send + Sync + 'static>(held: &mut Values) -> &mut T {
    held.downcast_mut::<T>()
        .expect("a guest callback is given the state its fitting held")
}

fn planted<T: Send + Sync + 'static>(fitting: Fitting<T>) -> Planted {
    Planted {
        name: std::any::type_name::<T>(),
        held: Box::new(fitting.held),
        ready: fitting.ready.map(|ready| -> Ready {
            Box::new(move |held, bench, resources| ready(typed_mut::<T>(held), bench, resources))
        }),
        draw: fitting.draw.map(|draw| -> Draw {
            Box::new(move |held, pass, canvas| draw(typed::<T>(held), pass, canvas))
        }),
    }
}

pub(crate) type Seed = Arc<dyn Fn(&mut Bench) -> Planted + Send + Sync>;

#[derive(Default)]
pub(crate) struct Guests {
    pub(crate) seeds: Vec<Seed>,
}

pub fn add_guest<T: Send + Sync + 'static>(
    resources: &mut Resources,
    seed: fn(&mut Bench) -> Fitting<T>,
) {
    hold::<Guests>(resources)
        .seeds
        .push(Arc::new(move |bench| planted(seed(bench))));
}
