use std::rc::Rc;

use web_sys::FocusEvent;
use yew::{html, Callback, Component, ComponentLink, Html, InputData, Properties, ShouldRender};
use yew_state::{SharedState, SharedStateComponent, Storable, StorageHandle};

type ViewForm<T> = Rc<dyn Fn(FormHandle<T>) -> Html>;

pub struct FormHandle<'a, T>
where
    T: Storable + Default + Clone + 'static,
{
    handle: &'a StorageHandle<T>,
}

impl<'a, T> FormHandle<'a, T>
where
    T: Storable + Default + Clone + 'static,
{
    /// Current form state.
    pub fn state(&self) -> &T {
        self.handle.state()
    }

    /// Callback that sets state, ignoring callback event.
    pub fn set<E: 'static>(&self, f: impl FnOnce(&mut T) + Copy + 'static) -> Callback<E> {
        self.handle.reduce_callback(f)
    }

    /// Callback that sets state from callback event
    pub fn set_with<E: 'static>(&self, f: impl FnOnce(&mut T, E) + Copy + 'static) -> Callback<E> {
        self.handle.reduce_callback_with(f)
    }

    /// Callback for settings state from `InputData`.
    pub fn set_input(
        &self,
        f: impl FnOnce(&mut T, String) + Copy + 'static,
    ) -> Callback<InputData> {
        self.handle
            .reduce_callback_with(f)
            .reform(|data: InputData| data.value)
    }
}

#[derive(Properties, Clone)]
pub struct Props<T>
where
    T: Storable + Default + Clone + 'static,
{
    #[prop_or_default]
    handle: StorageHandle<T>,
    #[prop_or_default]
    pub on_submit: Callback<T>,
    pub view: ViewForm<T>,
    // #[prop_or_default]
    // pub errors: InputErrors
}

impl<T> SharedState for Props<T>
where
    T: Storable + Default + Clone + 'static,
{
    type Handle = StorageHandle<T>;

    fn handle(&mut self) -> &mut Self::Handle {
        &mut self.handle
    }
}

pub enum Msg {
    Submit,
}

pub struct Model<T>
where
    T: Storable + Default + Clone + 'static,
{
    props: Props<T>,
    cb_submit: Callback<FocusEvent>,
}

impl<T> Component for Model<T>
where
    T: Storable + Default + Clone + 'static,
{
    type Message = Msg;
    type Properties = Props<T>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb_submit = link.callback(|e: FocusEvent| {
            e.prevent_default();
            Msg::Submit
        });
        Self { props, cb_submit }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Submit => {
                self.props.on_submit.emit(self.props.handle.state().clone());
                false
            }
        }
    }

    fn view(&self) -> Html {
        let handle = FormHandle {
            handle: &self.props.handle,
        };
        html! {
            <form onsubmit = self.cb_submit.clone()>
                { (self.props.view)(handle) }
            </form>
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

pub type Form<T> = SharedStateComponent<Model<T>>;

pub fn view_form<T: Storable + Default + Clone>(
    f: impl Fn(FormHandle<T>) -> Html + 'static,
) -> ViewForm<T> {
    Rc::new(f)
}
