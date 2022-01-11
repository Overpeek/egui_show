use egui::{Checkbox, DragValue, Slider, TextEdit, Ui};
use std::ops::RangeInclusive;

// traits

pub trait EguiShow {
    fn show(&mut self, ui: &mut Ui);
}

pub trait EguiShowValue
where
    Self: Sized,
{
    fn show(&mut self, ui: &mut Ui);
    fn show_range(&mut self, ui: &mut Ui, range: RangeInclusive<Self>);
}

// impl for numerics

pub trait Numeric {}
impl Numeric for f32 {}
impl Numeric for f64 {}
impl Numeric for u8 {}
impl Numeric for u16 {}
impl Numeric for u32 {}
impl Numeric for u64 {}
impl Numeric for usize {}
impl Numeric for i8 {}
impl Numeric for i16 {}
impl Numeric for i32 {}
impl Numeric for i64 {}
impl Numeric for isize {}

impl<T> EguiShowValue for T
where
    T: Numeric + egui::emath::Numeric,
{
    fn show(&mut self, ui: &mut Ui) {
        ui.add(DragValue::new(self));
    }

    fn show_range(&mut self, ui: &mut Ui, range: RangeInclusive<Self>) {
        ui.add(Slider::new(self, range));
    }
}

// impl for bool

impl EguiShowValue for bool {
    fn show(&mut self, ui: &mut Ui) {
        ui.add(Checkbox::new(self, ""));
    }

    fn show_range(&mut self, ui: &mut Ui, _: RangeInclusive<Self>) {
        self.show(ui);
    }
}

// impl for str

impl EguiShowValue for String {
    fn show(&mut self, ui: &mut Ui) {
        ui.add(TextEdit::singleline(self));
    }

    fn show_range(&mut self, ui: &mut Ui, _: RangeInclusive<Self>) {
        self.show(ui);
    }
}
