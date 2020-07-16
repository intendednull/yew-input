use web_sys::FocusEvent;
use yew::{html, Callback, Component, ComponentLink, Html, InputData, Properties, ShouldRender};
use yew_state::{GlobalHandle, SharedState, SharedStateComponent};
use yewtil::NeqAssign;

pub struct Setter<'a, T>
where
    T: Default + Clone + 'static,
{
    handle: &'a GlobalHandle<T>,
}

impl<'a, T> Setter<'a, T>
where
    T: Default + Clone + 'static,
{
    /// Callback that sets state, ignoring callback event.
    pub fn set<E: 'static>(&self, f: impl FnOnce(&mut T) + Copy + 'static) -> Callback<E> {
        self.handle.reduce_callback(f)
    }

    /// Callback that sets state from callback event
    pub fn from<E: 'static>(&self, f: impl FnOnce(&mut T, E) + Copy + 'static) -> Callback<E> {
        self.handle.reduce_callback_with(f)
    }

    /// Callback for settings state from an `InputData` event.
    pub fn from_input(
        &self,
        f: impl FnOnce(&mut T, String) + Copy + 'static,
    ) -> Callback<InputData> {
        self.handle
            .reduce_callback_with(f)
            .reform(|data: InputData| data.value)
    }
}

pub trait FormModel: Default + Clone {
    fn view(setter: Setter<Self>) -> Html;
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props<T>
where
    T: FormModel + PartialEq + 'static,
{
    #[prop_or_default]
    pub on_submit: Callback<T>,
    // #[prop_or_default]
    // pub errors: InputErrors,
    #[prop_or_default]
    handle: GlobalHandle<T>,
}

impl<T> SharedState for Props<T>
where
    T: FormModel + PartialEq + 'static,
{
    type Handle = GlobalHandle<T>;

    fn handle(&mut self) -> &mut Self::Handle {
        &mut self.handle
    }
}

pub enum Msg {
    Submit,
}

pub struct Model<T>
where
    T: FormModel + PartialEq + 'static,
{
    props: Props<T>,
    cb_submit: Callback<FocusEvent>,
}

impl<T> Component for Model<T>
where
    T: FormModel + PartialEq + 'static,
{
    type Message = Msg;
    type Properties = Props<T>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb_submit = link.callback(|e: FocusEvent| {
            e.prevent_default();
            Msg::Submit
        });
        // let setter = Rc::new(|)
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
        let setter = Setter {
            handle: &self.props.handle,
        };
        html! {
            <form onsubmit = self.cb_submit.clone()>
                { <T as FormModel>::view(setter) }
            </form>
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }
}

pub type Form<T> = SharedStateComponent<Model<T>>;
