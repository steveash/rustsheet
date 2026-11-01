#!/usr/bin/env python3
"""The merge train's shadow mode (SPEC-0027 section 4, "Shadow"; T-2995), run by
`scripts/merge_train.py --shadow`. Split out of merge_train.py, whose `Stage` it subclasses."""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass, field

import gh_client
import merge_ready
from gh_client import SHA, GhError
from merge_train import MAIN_REF, SHADOW, WINDOW, CutOff, Member, Stage, Stop, Train, read_mode, remote_sha, tree_of

# Shadow mode (SPEC-0027 section 4, "Shadow"; T-2995): steps 2-6 and section 6 against M0 on their own
# branch, with no lock and no merge. The predicted outcome of each PR goes into a `[skip ci]` commit on
# the shadow train, where `make merge-train-stats` (T-3076) reads it back through the commits API.
SHADOW_BRANCH = "merge-train/shadow"
PREDICTION_HEADER = "merge train shadow prediction [skip ci]"
NO_RESULT = "no result"


class Skip(Stop):
    """A shadow round that ends with nothing pushed: nothing changed since the last shadow train,
    or another shadow build won the race to the branch."""


def eject_reason(reason: str) -> str:
    """The `eject:<reason>` of a PR leaving the ready set: a conflict, an id collision, or a failed
    readiness check (`invalidated`). It reads the reasons that `build`, `id_collision` and `collect`
    write, so rewording one of those means updating this mapping and its test."""
    if reason == "conflict with main":
        return "eject:conflict"
    if reason.startswith("id collision"):
        return "eject:id-collision"
    return "eject:invalidated"


@dataclass
class ShadowStage(Stage):
    """One shadow round. It shares the train's build, CI wait and bisection, and swaps every write
    for a prediction: no lock, no merge, no label or comment but the closed-PR housekeeping, and no
    push but to `merge-train/shadow`."""

    branch: str = SHADOW_BRANCH
    predicted: dict[int, str] = field(default_factory=dict)
    inputs: str = ""
    tree_check: str = ""
    batch: list[int] = field(default_factory=list)
    pushed: bool = False

    mode = SHADOW

    def renew(self, prs: Sequence[int]) -> None:
        """Shadow mode holds no lock."""

    def stop_label(self) -> None:
        """Stop labels act in `train` mode only; shadow mode neither honours nor removes them."""

    def lost_lease(self) -> None:
        raise Skip(f"another shadow build won the race to {self.branch}; this round is skipped")

    def unlabel(self, pr: int, reason: str, *, eject: bool = False) -> None:
        self.predicted.setdefault(pr, eject_reason(reason))
        self.log.append(f"would {'eject' if eject else 'unlabel'} #{pr}: {reason}")

    def merge_green(self, train: Train, tip: str) -> list[int]:
        """A green (prefix of a) train is predicted to merge; nothing merges, so `main` stays M0."""
        for member in train.members:
            self.predicted.setdefault(member.pr, "merge")
        self.log.append("would merge " + ", ".join(f"#{m.pr}" for m in train.members) + f" (tested {tip[:12]})")
        return []

    def merge_prefix(self, train: Train, k: int, tip: str) -> str:
        """The predicted merge of a green prefix; the confirmation and the rest stay on M0, even
        when `main` moved during the round."""
        self.merge_green(Train(train.base, train.members[:k]), tip)
        return train.base

    def eject_red(self, member: Member, jobs: Sequence[str], links: str) -> None:
        self.predicted.setdefault(member.pr, "eject:ci-red")
        self.log.append(f"would eject #{member.pr}: ci-red ({'; '.join(jobs)}; runs: {links})")

    def collected(self, base: str, heads: Sequence[tuple[int, str]]) -> None:
        """Skips the round when neither `main` nor the ready set (PRs and heads) changed since the
        last shadow train, whose commits name their input, so CI is not spent on repeats."""
        self.batch = [pr for pr, _ in heads]
        self.inputs = f"shadow-input: {base} " + " ".join(f"#{pr} {head}" for pr, head in heads)
        if self.lease and self.inputs in merge_ready.text_of(self.client.commit(self.lease), "message").splitlines():
            raise Skip("main and the ready set are unchanged since the last shadow train")

    def build_on(self, base: str, heads: Sequence[tuple[int, str]]) -> Train:
        train = super().build_on(base, heads)
        for pr, _ in train.deferred:
            self.predicted.setdefault(pr, "defer")
        if not self.tree_check and train.members:
            self.tree_check = self.check_tree(train)
        return train

    def check_tree(self, train: Train) -> str:
        """The shadow tree check: the first PR's GitHub test merge sits on M0 while `main` has not
        moved, so its tree must be `tree(T_1)`. `n/a` when `main` moved or there is no test merge."""
        first = train.members[0]
        merge = merge_ready.text_of(self.client.pull(first.pr), "merge_commit_sha")
        if not SHA.fullmatch(merge) or remote_sha(MAIN_REF, self.root) != train.base:
            return "n/a"
        commit = self.client.commit(merge)
        parents = merge_ready.field_of(commit, "parents")
        parent = parents[0] if isinstance(parents, list) and parents else None
        if merge_ready.text_of(parent, "sha") != train.base:
            return "n/a"
        return "match" if merge_ready.text_of(commit, "tree", "sha") == tree_of(first.merge, self.root) else "mismatch"

    def push(self, train: Train, notes: Sequence[str] = ()) -> str:
        tip = super().push(train, [*notes, self.inputs])
        self.pushed = True
        return tip

    def housekeeping(self) -> None:
        """The one PR-visible write of shadow mode: `ready-to-merge` comes off closed PRs, which a
        worker merged by the per-PR flow. A failure is logged and the round goes on: the
        housekeeping is not what the round tests."""
        try:
            for item in self.client.closed_labelled(merge_ready.READY_LABEL):
                if merge_ready.field_of(item, "pull_request") is None:
                    continue
                pr = gh_client.number(merge_ready.int_of(item, "number"))
                self.client.remove_label(pr, merge_ready.READY_LABEL)
                self.log.append(f"removed {merge_ready.READY_LABEL} from closed #{pr}")
        except GhError as error:
            self.log.append(f"housekeeping failed: {error}")

    def prediction(self) -> str:
        """The predicted-outcome record: each PR's prediction (`no result` where a stopped round
        decided nothing), the would-be unlabels and the tree check."""
        lines = [PREDICTION_HEADER, "", self.inputs]
        for pr in dict.fromkeys([*self.batch, *self.predicted]):
            lines.append(f"#{pr} {self.predicted.get(pr, NO_RESULT)}")
        unlabel = [pr for pr, outcome in self.predicted.items() if outcome != "defer"]
        lines.append("unlabel: " + (", ".join(f"#{pr}" for pr in unlabel) or "none"))
        lines.append(f"tree-check: {self.tree_check or 'n/a'}")
        return "\n".join(lines)

    def finish(self) -> None:
        """Records the prediction on the shadow train this round pushed; a lost lease skips it."""
        if not self.pushed:
            return
        try:
            self.write_train_ref(tree_of(self.lease, self.root), self.lease, self.prediction())
        except Skip as skip:
            self.log.append(f"skipped: {skip}")
            return
        self.log.append(f"recorded the prediction on {self.branch}")

    def run(self, window: float = WINDOW) -> int:
        """One shadow round in `shadow` mode. 0 when it ended normally or was skipped, 1 when it
        stopped. A cut-off keeps its search-state record as the branch tip, so the next round
        resumes the bisection; any other stop records `no result` for the PRs it did not decide."""
        mode = read_mode(self.root)
        if mode != SHADOW:
            self.log.append(f"the mode on main is {mode}: shadow mode stops")
            return 0
        self.housekeeping()
        try:
            self.hold()
        except Skip as skip:
            self.log.append(f"skipped: {skip}")
            return 0
        except CutOff as stop:
            self.log.append(f"stopped: {stop}")
            return 1
        except Stop as stop:
            self.log.append(f"stopped: {stop}")
            self.finish()
            return 1
        self.finish()
        return 0
