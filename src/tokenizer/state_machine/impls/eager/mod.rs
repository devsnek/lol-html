#[macro_use]
mod actions;
mod conditions;

use base::{Align, Chunk, Cursor, Range};
use std::cell::RefCell;
use std::rc::Rc;
use tokenizer::outputs::*;
use tokenizer::state_machine::{
    ParsingLoopDirective, ParsingLoopTerminationReason, StateMachine, StateMachineBookmark,
    StateResult,
};
use tokenizer::tree_builder_simulator::*;
use tokenizer::{NextOutputType, TagName, TagPreviewHandler, TextParsingMode};

#[cfg(feature = "testing_api")]
use tokenizer::TextParsingModeChangeHandler;

pub type State<H> = fn(&mut EagerStateMachine<H>, &Chunk) -> StateResult;

pub struct EagerStateMachine<H: TagPreviewHandler> {
    input_cursor: Cursor,
    tag_start: Option<usize>,
    tag_name_start: usize,
    is_in_end_tag: bool,
    tag_name_hash: Option<u64>,
    last_start_tag_name_hash: Option<u64>,
    state_enter: bool,
    allow_cdata: bool,
    tag_preview_handler: H,
    state: State<H>,
    closing_quote: u8,
    tree_builder_simulator: Rc<RefCell<TreeBuilderSimulator>>,
}

impl<H: TagPreviewHandler> EagerStateMachine<H> {
    pub fn new(
        tag_preview_handler: H,
        tree_builder_simulator: &Rc<RefCell<TreeBuilderSimulator>>,
    ) -> Self {
        EagerStateMachine {
            input_cursor: Cursor::default(),
            tag_start: None,
            tag_name_start: 0,
            is_in_end_tag: false,
            tag_name_hash: None,
            last_start_tag_name_hash: None,
            state_enter: true,
            allow_cdata: false,
            tag_preview_handler,
            state: EagerStateMachine::data_state,
            closing_quote: b'"',
            tree_builder_simulator: Rc::clone(tree_builder_simulator),
        }
    }

    // TODO move to trait and rename to create bookmark
    fn create_output_type_switch_loop_directive(&self, pos: usize) -> ParsingLoopDirective {
        let bookmark = StateMachineBookmark {
            allow_cdata: self.allow_cdata,
            text_parsing_mode: TextParsingMode::Data, // TODO
            last_start_tag_name_hash: self.last_start_tag_name_hash,
            pos,
        };

        ParsingLoopDirective::Break(ParsingLoopTerminationReason::OutputTypeSwitch(bookmark))
    }
}

impl<H: TagPreviewHandler> StateMachine for EagerStateMachine<H> {
    #[inline]
    fn set_state(&mut self, state: State<H>) {
        self.state = state;
    }

    #[inline]
    fn get_state(&self) -> State<H> {
        self.state
    }

    #[inline]
    fn get_input_cursor(&mut self) -> &mut Cursor {
        &mut self.input_cursor
    }

    #[inline]
    fn get_blocked_byte_count(&self, input: &Chunk) -> usize {
        input.len() - self.tag_start.unwrap_or(0)
    }

    fn adjust_for_next_input(&mut self) {
        if let Some(tag_start) = self.tag_start {
            self.input_cursor.align(tag_start);
            self.tag_name_start.align(tag_start);
            self.tag_start = Some(0);
        }
    }

    #[inline]
    fn set_is_state_enter(&mut self, val: bool) {
        self.state_enter = val;
    }

    #[inline]
    fn is_state_enter(&self) -> bool {
        self.state_enter
    }

    #[inline]
    fn get_closing_quote(&self) -> u8 {
        self.closing_quote
    }

    #[cfg(feature = "testing_api")]
    fn set_last_start_tag_name_hash(&mut self, name_hash: Option<u64>) {
        self.last_start_tag_name_hash = name_hash;
    }

    #[cfg(feature = "testing_api")]
    fn set_text_parsing_mode_change_handler(
        &mut self,
        _handler: Box<dyn TextParsingModeChangeHandler>,
    ) {
        // Noop
    }
}