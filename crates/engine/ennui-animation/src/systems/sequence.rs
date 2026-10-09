use crate::commands::sequence::{bind, clock, drive, pose, release, send_cues};
use crate::components::{Channel, Play, Sequence};
use crate::data::{EPSILON, Ended, Marked, Named};
use crate::queries::ease::strand_of;
use crate::queries::target::clip_of;
use ennui::events::send;
use ennui::prelude::{Edits, Events, Mut, Peek, Res, ResMut, View, each, each_mut, peek};
use ennui::reflect::prelude::Reflected;
use ennui::system::{ticked, touched};
use ennui_document::prelude::{Placed, SceneId};
use ennui_platform::prelude::Time;
use ennui_scene::prelude::ChildOf;

pub(crate) fn play_sequences(
    mut players: Mut<(Play,)>,
    channels: View<(&Channel, &ChildOf)>,
    channel_ticks: Peek<Channel>,
    sequences: Peek<Sequence>,
    ids: Peek<SceneId>,
    listed: View<(&SceneId,)>,
    parents: Peek<ChildOf>,
    time: Res<Time>,
    registry: Res<Reflected>,
    placed: Res<Placed>,
    mut edits: Edits,
    mut marks: ResMut<Events<Marked>>,
    mut ended: ResMut<Events<Ended>>,
) {
    let step = time.step;
    let now = ticked(&sequences);
    let named = Named {
        ids: &listed,
        parents: &parents,
    };
    each_mut(&mut players, |root, (mut held,), ()| {
        let play: &mut Play = &mut held;
        let root_id = peek(&ids, root).map(|id| id.0.as_str());
        let Some((clip, sequence)) = clip_of(&placed, &named, &sequences, root, root_id, play)
        else {
            release(&mut edits, play);
            play.clip_entity = None;
            return;
        };
        if play.clip_entity != Some(clip)
            || touched(&sequences, play.since)
            || touched(&channel_ticks, play.since)
        {
            release(&mut edits, play);
            let owned = each(&channels)
                .filter(|(_, (_, of))| of.0 == clip)
                .map(|(_, (channel, _))| channel);
            bind(play, owned, &registry, (&placed, &named), root, root_id);
            play.clip_entity = Some(clip);
            play.since = now;
        }
        let length = sequence.length.max(EPSILON);
        let looping = play.looping || sequence.looping;
        let done = clock(play, length, looping, step);
        let strand = strand_of(play, looping);
        send_cues(&mut marks, sequence, strand, length, root);
        for bound in play.bound.iter_mut() {
            pose(bound, play.time);
        }
        drive(&mut edits, root);
        if done {
            play.playing = false;
            send(&mut ended, Ended { entity: root });
        }
    });
}
