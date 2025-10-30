use std::marker::PhantomData;

use bevy::prelude::*;

#[derive(Component, Default)]
pub struct RequestBuffer<T: Send + Sync + 'static>(pub Vec<T>);

#[derive(Component, Debug)]
pub struct FinalDecision<R: Send + Sync + 'static>(pub R);

#[derive(Component, Debug)]
pub struct LastDecision<R: Send + Sync + 'static>(pub R);

pub trait PipelineStages: SystemSet + Sized {
    fn modify() -> Self;

    fn reduce() -> Self;

    fn apply() -> Self;

    fn cleanup() -> Self;
}

pub struct ArbitrationPipelinePlugin<T, P>
where
    T: Event + Clone + Send + Sync + 'static,
    P: PipelineStages + Send + Sync + 'static,
{
    _intent: PhantomData<T>,
    _pipeline: PhantomData<P>,
}

impl<T, P> Default for ArbitrationPipelinePlugin<T, P>
where
    T: Event + Clone + Send + Sync + 'static,
    P: PipelineStages + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            _intent: PhantomData,
            _pipeline: PhantomData,
        }
    }
}

impl<T, P> Plugin for ArbitrationPipelinePlugin<T, P>
where
    T: Event + Clone + Send + Sync + 'static,
    P: PipelineStages + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedPostUpdate,
            (P::modify(), P::reduce(), P::apply(), P::cleanup()).chain(),
        );

        app.add_observer(accumulate_requests::<T, P>);

        app.add_systems(
            FixedPostUpdate,
            (cleanup_buffer::<T, T, P>, cleanup_result::<T, T, P>).in_set(P::cleanup()),
        );
    }
}

fn accumulate_requests<T, P>(
    trigger: Trigger<T>,
    mut commands: Commands,
    mut query: Query<(Entity, Option<&mut RequestBuffer<T>>)>,
) where
    T: Event + Clone + Send + Sync + 'static,
    P: PipelineStages,
{
    let entity = trigger.target();
    let event = trigger.event();

    if let Ok((_e, buffer_opt)) = query.get_mut(entity) {
        if let Some(mut buffer) = buffer_opt {
            buffer.0.push(event.clone());
        } else {
            commands
                .entity(entity)
                .insert(RequestBuffer(vec![event.clone()]));
        }
    }
}

fn cleanup_buffer<T, R, P>(
    mut commands: Commands,
    mut buffer_query: Query<(Entity, &mut RequestBuffer<T>)>,
) where
    T: Send + Sync + 'static,
    R: Send + Sync + 'static,
    P: PipelineStages,
{
    for (entity, mut buffer) in buffer_query.iter_mut() {
        if buffer.0.is_empty() {
            commands.entity(entity).remove::<RequestBuffer<T>>();
        } else {
            buffer.0.clear();
        }
    }
}

fn cleanup_result<T, R, P>(
    mut commands: Commands,

    decision_query: Query<(Entity, &FinalDecision<R>)>,
) where
    T: Send + Sync + 'static,
    R: Clone + Send + Sync + 'static,
    P: PipelineStages,
{
    for (entity, final_decision) in decision_query.iter() {
        commands
            .entity(entity)
            .insert(LastDecision(final_decision.0.clone()))
            .remove::<FinalDecision<R>>();
    }
}
