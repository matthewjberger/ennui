use crate::data::Reflect;
use crate::queries::kind::refusal;
use crate::queries::names::short_name;
use crate::resources::Settings;
use ennui_ecs::system::ResMut;

pub fn settle<R: Send + Sync + Reflect + 'static>(
    mut settings: ResMut<Settings>,
    mut held: ResMut<R>,
) {
    let name = short_name(std::any::type_name::<R>());
    if !settings.made.contains_key(name) {
        settings.made.insert(String::from(name), R::value_of(&held));
    }
    if settings.reset.remove(name)
        && let Some(made) = settings.made.get(name)
    {
        R::apply(&mut held, made);
    }
    let wanted: Vec<_> = settings
        .wanted
        .extract_if(.., |(held, _)| held == name)
        .map(|(_, value)| value)
        .collect();
    for value in wanted {
        if !R::apply(&mut held, &value) {
            settings.problems.push(refusal(name, &R::fields(), &value));
        }
    }
    let now = R::value_of(&held);
    if settings.shown.get(name) != Some(&now) {
        settings.shown.insert(String::from(name), now);
        settings.turn += 1;
        let turn = settings.turn;
        settings.changed.insert(String::from(name), turn);
    }
}
