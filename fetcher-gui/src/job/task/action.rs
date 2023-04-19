/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod contains;
pub mod decode_html;
pub mod extract;
pub mod field;
pub mod html;
pub mod json;
pub mod remove_html;
pub mod replace;
pub mod set;
pub mod shorten;
pub mod sink;
pub mod take;
pub mod use_as;

use self::{
	contains::ContainsState, json::JsonState, set::SetState, shorten::ShortenState,
	sink::SinkState, take::TakeState, use_as::UseState,
};
use fetcher_config::jobs::{
	action::{
		contains::Contains, decode_html::DecodeHtml, extract::Extract, html::Html, import::Import,
		json::Json, remove_html::RemoveHtml, replace::Replace, set::Set, shorten::Shorten,
		take::Take, trim::Trim, use_as::Use, Action,
	},
	sink::Sink,
};

use egui::{
	panel::Side, CentralPanel, ComboBox, ScrollArea, SelectableLabel, SidePanel, TopBottomPanel, Ui,
};
use std::{collections::HashMap, hash::Hash};

#[derive(Default, Debug)]
pub struct ActionEditorState {
	pub current_action_idx: Option<usize>,
	pub selected_action_state: HashMap<usize, SelectedActionState>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum SelectedActionState {
	Stateless,
	TakeState(TakeState),
	ContainsState(ContainsState),
	JsonState(JsonState),
	UseState(UseState),
	SetState(SetState),
	ShortenState(ShortenState),
	SinkState(SinkState),
}

impl ActionEditorState {
	pub fn show(&mut self, actions: &mut Option<Vec<Action>>, task_id: impl Hash, ui: &mut Ui) {
		SidePanel::new(Side::Left, egui::Id::new(("actions list", &task_id)))
			.show_inside(ui, |ui| self.side_panel(actions, &task_id, ui));

		// NOTE: fixes a bug in egui that makes the CentralPanel below overflow the window.
		// See https://github.com/emilk/egui/issues/901
		TopBottomPanel::bottom(egui::Id::new((
			"actions list invisible bottom panel",
			&task_id,
		)))
		.show_separator_line(false)
		.show_inside(ui, |_| ());

		CentralPanel::default().show_inside(ui, |ui| {
			ScrollArea::vertical().show(ui, |ui| {
				if let Some((idx, act)) = actions
					.as_mut()
					.zip(self.current_action_idx)
					.and_then(|(actions, idx)| Some((idx, actions.get_mut(idx)?)))
				{
					self.selected_action_state
						.entry(idx)
						.or_insert_with(|| SelectedActionState::new(act))
						.show(act, task_id, ui);
				}
			});
		});
	}

	/// action list side panel + add button
	pub fn side_panel(
		&mut self,
		actions: &mut Option<Vec<Action>>,
		task_id: impl Hash,
		ui: &mut Ui,
	) {
		TopBottomPanel::bottom(egui::Id::new(("action list add button panel", &task_id)))
			.show_separator_line(false)
			.show_inside(ui, |ui| {
				ComboBox::from_id_source(("action list add button", &task_id))
					.selected_text("Add")
					.width(ui.available_width())
					.show_ui(ui, |ui| {
						/// Creates ui.selectable_label's for provided actions that pushes the action with the default state to the actions list
						macro_rules! add_button {
						    ($(Action::$act:ident$( => $default:expr)?),+) => {
								$({
									if ui.selectable_label(false, stringify!($act)).clicked() {
										actions
											.get_or_insert_with(Vec::new)
											.push(Action::$act$(($default))?);	// push either Action::$act (for unit variants) or Action::$act($default) if the => $default arm is present
									}
								})+
						    };
						}

						add_button! {
							Action::ReadFilter,
							Action::Take => Take::default(),
							Action::Contains => Contains::default(),
							Action::DebugPrint,
							Action::Feed,
							Action::Html => Html::default(),
							Action::Http,
							Action::Json => Json::default(),
							Action::Use => Use::default(),
							Action::Caps,
							Action::Set => Set::default(),
							Action::Shorten => Shorten::default(),
							Action::Trim => Trim::default(),
							Action::Replace => Replace::default(),
							Action::Extract => Extract::default(),
							Action::RemoveHtml => RemoveHtml::default(),
							Action::DecodeHtml => DecodeHtml::default(),
							Action::Sink => Sink::default(),
							Action::Import => Import::default()
						};
					});
			});

		CentralPanel::default().show_inside(ui, |ui| {
			ScrollArea::vertical().show(ui, |ui| {
				for (idx, act) in actions.iter().flatten().enumerate() {
					// TODO: left align the text
					if ui
						.add_sized(
							[ui.available_width(), 0.0],
							SelectableLabel::new(
								*self.current_action_idx.get_or_insert(0) == idx,
								act.to_string(),
							),
						)
						.clicked()
					{
						self.current_action_idx = Some(idx);
					}
				}
			});
		});
	}
}

impl SelectedActionState {
	pub fn new(for_action: &Action) -> Self {
		match for_action {
			Action::ReadFilter => Self::Stateless,
			Action::Take(_) => Self::TakeState(Default::default()),
			Action::Contains(_) => Self::ContainsState(Default::default()),
			Action::DebugPrint => Self::Stateless,
			Action::Feed => Self::Stateless,
			Action::Html(_) => Self::Stateless,
			Action::Http => Self::Stateless,
			Action::Json(_) => Self::JsonState(Default::default()),
			Action::Use(_) => Self::UseState(Default::default()),
			Action::Caps => Self::Stateless,
			Action::Set(_) => Self::SetState(Default::default()),
			Action::Shorten(_) => Self::ShortenState(Default::default()),
			Action::Trim(_) => Self::Stateless,
			Action::Replace(_) => Self::Stateless,
			Action::Extract(_) => Self::Stateless,
			Action::RemoveHtml(_) => Self::Stateless,
			Action::DecodeHtml(_) => Self::Stateless,
			Action::Sink(_) => Self::SinkState(Default::default()),
			Action::Import(_) => Self::Stateless,
		}
	}

	pub fn show(&mut self, action: &mut Action, task_id: impl Hash, ui: &mut Ui) {
		match (&mut *self, &mut *action) {
			(Self::Stateless, Action::ReadFilter) => (),
			(Self::TakeState(state), Action::Take(x)) => state.show(x, &task_id, ui),
			(Self::ContainsState(state), Action::Contains(x)) => state.show(x, &task_id, ui),
			(Self::Stateless, Action::DebugPrint) => (),
			(Self::Stateless, Action::Feed) => (),
			(Self::Stateless, Action::Html(x)) => html::show(x, &task_id, ui),
			(Self::Stateless, Action::Http) => (),
			(Self::JsonState(state), Action::Json(x)) => state.show(x, &task_id, ui),
			(Self::UseState(state), Action::Use(x)) => state.show(x, &task_id, ui),
			(Self::Stateless, Action::Caps) => (),
			(Self::SetState(state), Action::Set(x)) => state.show(x, &task_id, ui),
			(Self::ShortenState(state), Action::Shorten(x)) => state.show(x, &task_id, ui),
			(Self::Stateless, Action::Trim(x)) => field::show(&mut x.field, &task_id, ui),
			(Self::Stateless, Action::Replace(x)) => replace::show(x, &task_id, ui),
			(Self::Stateless, Action::Extract(x)) => extract::show(x, &task_id, ui),
			(Self::Stateless, Action::RemoveHtml(x)) => remove_html::show(x, &task_id, ui),
			(Self::Stateless, Action::DecodeHtml(x)) => decode_html::show(x, &task_id, ui),
			(Self::SinkState(state), Action::Sink(x)) => state.show(x, &task_id, ui),
			(Self::Stateless, Action::Import(x)) => {
				ui.text_edit_singleline(&mut x.0);
			}
			// state doesn't match the action, create a new one
			_ => {
				/*
				// TODO: will create an infinite loop if no match arms still match. Create a check to avoid that
				*self = Self::new(action);
				self.show(action, task_id, ui);
				*/
				todo!();
			}
		}
	}
}
