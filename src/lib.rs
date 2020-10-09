use std::rc::Rc;

use web_sys::{Event, FocusEvent, HtmlElement};
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::services::Task;
use yew::{
    html, Callback, ChangeData, Component, ComponentLink, Html, InputData, NodeRef, Properties,
    ShouldRender,
};
use yew_state::{SharedHandle, SharedState, SharedStateComponent};

type ViewForm<T> = Rc<dyn Fn(FormHandle<T>) -> Html>;

pub struct FormHandle<'a, T>
where
    T: PartialEq + Default + Clone + 'static,
{
    handle: &'a SharedHandle<T>,
    link: &'a ComponentLink<Model<T>>,
    ref_form: &'a NodeRef,
}

impl<'a, T> FormHandle<'a, T>
where
    T: PartialEq + Default + Clone + 'static,
{
    /// Current form state.
    pub fn state(&self) -> &T {
        self.handle.state()
    }

    /// Callback for submitting the form.
    pub fn submit<E: 'static>(&self) -> Callback<E> {
        let node = self.ref_form.clone();
        let submit = move |_| {
            if let Some(el) = node.cast::<HtmlElement>() {
                let event = FocusEvent::new("submit").unwrap();
                el.dispatch_event(&event).unwrap();
            }
        };
        submit.into()
    }

    /// Callback that sets state, ignoring callback event.
    pub fn set<E: 'static>(&self, f: impl FnOnce(&mut T) + 'static) -> Callback<E> {
        self.handle.reduce_callback_once(f)
    }

    /// Callback that sets state from callback event
    pub fn set_with<E: 'static>(&self, f: impl FnOnce(&mut T, E) + 'static) -> Callback<E> {
        self.handle.reduce_callback_once_with(f)
    }

    /// Callback for setting state from `InputData`.
    pub fn set_text(&self, f: impl FnOnce(&mut T, String) + 'static) -> Callback<InputData> {
        self.handle
            .reduce_callback_once_with(f)
            .reform(|data: InputData| data.value)
    }

    /// Callback for setting state from select elements.
    ///
    /// # Panics
    ///
    /// Panics if used on anything other than a select element.
    pub fn set_select(&self, f: impl FnOnce(&mut T, String) + 'static) -> Callback<ChangeData> {
        self.handle
            .reduce_callback_once_with(f)
            .reform(|data: ChangeData| {
                if let ChangeData::Select(el) = data {
                    el.value()
                } else {
                    panic!("Select element is required")
                }
            })
    }

    /// Callback for setting files
    pub fn set_file(
        &self,
        f: impl FnOnce(&mut T, FileData) + Copy + 'static,
    ) -> Callback<ChangeData> {
        let set_files = self.set_with(f);
        self.link.callback(move |data| {
            let mut result = Vec::new();
            if let ChangeData::Files(files) = data {
                let files = js_sys::try_iter(&files)
                    .unwrap()
                    .unwrap()
                    .into_iter()
                    .map(|v| File::from(v.unwrap()));
                result.extend(files);
            }
            Msg::Files(result, set_files.clone())
        })
    }
}

#[derive(Properties, Clone)]
pub struct Props<T>
where
    T: PartialEq + Default + Clone + 'static,
{
    #[prop_or_default]
    handle: SharedHandle<T>,
    #[prop_or_default]
    pub on_submit: Callback<T>,
    #[prop_or_default]
    pub default: Option<T>,
    #[prop_or_default]
    pub auto_reset: bool,
    pub view: ViewForm<T>,
}

impl<T> SharedState for Props<T>
where
    T: PartialEq + Default + Clone + 'static,
{
    type Handle = SharedHandle<T>;

    fn handle(&mut self) -> &mut Self::Handle {
        &mut self.handle
    }
}

pub enum Msg {
    Files(Vec<File>, Callback<FileData>),
    Submit(FocusEvent),
}

pub struct Model<T>
where
    T: PartialEq + Default + Clone + 'static,
{
    props: Props<T>,
    cb_submit: Callback<FocusEvent>,
    cb_reset: Callback<()>,
    link: ComponentLink<Self>,
    file_reader: ReaderService,
    tasks: Vec<ReaderTask>,
    ref_form: NodeRef,
}

impl<T> Component for Model<T>
where
    T: PartialEq + Default + Clone + 'static,
{
    type Message = Msg;
    type Properties = Props<T>;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb_submit = link.callback(|e: FocusEvent| {
            e.prevent_default();
            Msg::Submit(e)
        });
        let mut this = Self {
            props,
            cb_submit,
            link,
            cb_reset: Default::default(),
            tasks: Default::default(),
            file_reader: Default::default(),
            ref_form: Default::default(),
        };
        this.update_default();
        this
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Submit(e) => {
                self.props.on_submit.emit(self.props.handle.state().clone());
                if self.props.auto_reset {
                    // Clear form
                    let reset_event = Event::new("reset").unwrap();
                    e.target()
                        .map(|target| target.dispatch_event(&reset_event).ok());
                    // Reset state
                    self.cb_reset.emit(());
                }
                false
            }
            Msg::Files(files, cb) => {
                self.tasks.retain(Task::is_active);
                for file in files.into_iter() {
                    let task = self
                        .file_reader
                        .read_file(file, cb.clone())
                        .expect("Error reading file");

                    self.tasks.push(task);
                }
                false
            }
        }
    }

    fn view(&self) -> Html {
        let handle = FormHandle {
            handle: &self.props.handle,
            link: &self.link,
            ref_form: &self.ref_form,
        };
        html! {
            <form
                ref=self.ref_form.clone()
                onreset = self.cb_reset.reform(|_| ())
                onsubmit = self.cb_submit.clone()>
                { (self.props.view)(handle) }
            </form>
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let last = self.props.default.clone();
        self.props = props;
        if self.props.default != last {
            self.update_default();
        }
        true
    }
}

impl<T> Model<T>
where
    T: PartialEq + Default + Clone + 'static,
{
    fn update_default(&mut self) {
        let default = self
            .props
            .default
            .as_ref()
            .map(Clone::clone)
            .unwrap_or_default();

        self.cb_reset = self
            .props
            .handle()
            .reduce_callback(move |state| *state = default.clone());

        // Make sure default is set if provided.
        if self.props.default.is_some() {
            self.cb_reset.emit(());
        }
    }
}

pub struct FormScope;
pub type Form<T, SCOPE = FormScope> = SharedStateComponent<Model<T>, SCOPE>;

pub fn view_form<T: PartialEq + Default + Clone>(
    f: impl Fn(FormHandle<T>) -> Html + 'static,
) -> ViewForm<T> {
    Rc::new(f)
}
