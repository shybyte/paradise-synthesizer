use midi_message::Note;

pub(crate) struct PressedNotes {
    notes: Vec<Note>,
}

impl PressedNotes {
    pub fn new() -> Self {
        PressedNotes { notes: vec![] }
    }

    pub fn get_current_note(&self) -> Option<Note> {
        self.notes.last().copied()
    }

    pub fn note_on(&mut self, note: Note) {
        self.notes.push(note);
    }

    pub fn note_off(&mut self, note: Note) {
        self.notes.retain(|it| *it != note);
    }
}
