//! The diff lists two models exchange to stay in sync: [`UserModel::flush_send_queue`] hands
//! out the edits made here, [`UserModel::apply_external_diffs`] applies a peer's.

use crate::{
    user_model::history::{DiffType, QueueDiffs},
    workbook_check::SpillTally,
    Model, UserModel,
};

impl UserModel {
    /// Returns the list of pending diffs and removes them from the queue
    ///
    /// This is used together with [`apply_external_diffs`](UserModel::apply_external_diffs) to keep two remote models
    /// in sync.
    ///
    /// See also:
    /// * [`UserModel::apply_external_diffs`]
    pub fn flush_send_queue(&mut self) -> Vec<u8> {
        // This can never fail :O:
        let q = bitcode::encode(&self.send_queue);
        self.send_queue = vec![];
        q
    }

    /// This are external diffs that need to be applied to the model
    ///
    /// This is used together with [`flush_send_queue`](UserModel::flush_send_queue) to keep two remote models in sync
    ///
    /// The bytes come from a peer, so they are untrusted (CLAUDE.md directive 5, T-2896): each
    /// diff is checked against the model as it is when the diff applies, and a list that is
    /// refused anywhere leaves the model as it was before the first diff applied.
    ///
    /// See also:
    /// * [`UserModel::flush_send_queue`]
    pub fn apply_external_diffs(&mut self, diff_list_str: &[u8]) -> Result<(), String> {
        let queue_diffs_list = bitcode::decode::<Vec<QueueDiffs>>(diff_list_str)
            .map_err(|_| "Error parsing diff list".to_string())?;
        // Whether a diff applies depends on the ones before it (a sheet the list adds, a style
        // an edit creates), so the list is applied to the model and undone by restoring this
        // copy if a diff is refused. The copy costs a pass over the workbook on every call; a list
        // that changes a cell already evaluates the whole workbook.
        let before = self.model.workbook.clone();
        self.restored_spills = SpillTally::default();
        let applied = queue_diffs_list.iter().try_for_each(|queue_diff| {
            if matches!(queue_diff.r#type, DiffType::Redo) {
                self.apply_diff_list(&queue_diff.list)
            } else {
                self.apply_undo_diff_list(&queue_diff.list)
            }
        });
        if let Err(refusal) = applied {
            let language = self.model.language.code.clone();
            let clock = self.model.clock;
            self.model = Model::from_workbook_with_clock(before, &language, clock)
                .map_err(|e| format!("{refusal}; the model could not be restored: {e}"))?;
            self.evaluate_if_not_paused();
            return Err(refusal);
        }
        Ok(())
    }
}
