#[allow(dead_code)]
use bevy_ecs::prelude::*;
use rand::{ thread_rng, Rng };
use integer_sqrt::IntegerSquareRoot;
use std::time::Instant;
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;

#[derive(Component, PartialEq, Eq, Copy, Clone, Debug, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn distance(self, p: Position) -> i32 {
        let x = self.x - p.x;
        let y = self.y - p.y;

        (x * x + y * y).integer_sqrt()
    }
}

#[derive(Component, PartialEq, Eq, Default)]
pub struct State(bool);

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 { write!(f, "Alive") } else { write!(f, "Dead") }
    }
}

#[derive(Resource, Debug)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
}

#[derive(Component, Debug, Default, PartialEq, Eq)]
pub struct Neighbors(u8);

#[derive(Resource)]
pub struct Generations(u32);

const K_DIMENSIONS: usize = 2;

#[derive(Resource)]
struct KdTreeResource {
    kd_tree: KdTree<f64, (i32, i32), [f64; K_DIMENSIONS]>,
}

#[derive(Resource)]
struct CellsChanged(bool);

#[derive(Component)]
pub struct Alive;

#[derive(Component)]
pub struct Dead;

#[derive(Bundle, Default)]
pub struct CellBundle {
    pub position: Position,
    pub state: State,
    pub neighbors: Neighbors,
}

fn rebuild_kd_tree(
    query: Query<(&Position, &State)>,
    mut kd_tree_resource: ResMut<KdTreeResource>,
    mut cells_changed: ResMut<CellsChanged>
) {
    if !cells_changed.0 {
        return;
    }

    let start = Instant::now();
    kd_tree_resource.kd_tree = KdTree::new(K_DIMENSIONS);
    for (pos, state) in query.iter() {
        if state.0 {
            let point = [pos.x as f64, pos.y as f64];
            kd_tree_resource.kd_tree.add(point, (pos.x, pos.y)).unwrap();
        }
    }

    cells_changed.0 = false;

    let duration = start.elapsed();
    println!("Building kd tree took {:?}", duration);
}

// Cell entity - cell is a tuple of Position, State, and Neighbors

pub fn spawn_cells(world: &mut World, width: u32, height: u32) {
    let start = Instant::now();
    let cells_to_spawn_count = width * height;
    let to_spawn = (0..cells_to_spawn_count).map(|i| {
        let x = i % width;
        let y = i / width;
        let position = Position { x: x as i32, y: y as i32 };
        let mut rng = thread_rng();
        let state = State(rng.gen_bool(0.5));
        CellBundle {
            position,
            state,
            ..Default::default()
        }
    });

    world.spawn_batch(to_spawn);
    println!("Spawning {:?} cells", cells_to_spawn_count);
    let duration = start.elapsed();
    println!("Spawning cells took {:?}", duration);
}

pub fn spawn_block_cells(world: &mut World, width: u32, height: u32) {
    let start = Instant::now();
    let cells_to_spawn_count = width * height;
    let to_spawn = (0..cells_to_spawn_count).map(|i| {
        let x = i % width;
        let y = i / width;
        let position = Position { x: x as i32, y: y as i32 };
        let state = State(true);
        println!("Spawning block cell at position {:?}, with state {:?}", position, state);
        CellBundle {
            position,
            state,
            ..Default::default()
        }
    });

    world.spawn_batch(to_spawn);
    println!("Spawning {:?} cells", cells_to_spawn_count);
    let duration = start.elapsed();
    println!("Spawning cells took {:?}", duration);
}

pub fn spawn_beehive_cells(world: &mut World, width: u32, height: u32) {
    let start = Instant::now();
    let cells_to_spawn_count = width * height;
    let to_spawn = (0..cells_to_spawn_count).map(|i| {
        let x = i % width;
        let y = i / width;
        let position = Position { x: x as i32, y: y as i32 };
        let state = match (x, y) {
            (2, 0) => State(true),
            (3, 0) => State(true),
            (1, 1) => State(true),
            (4, 1) => State(true),
            (2, 2) => State(true),
            (3, 2) => State(true),
            _ => State(false),
        };
        CellBundle {
            position,
            state,
            ..Default::default()
        }
    });

    world.spawn_batch(to_spawn);
    println!("Spawning {:?} cells", cells_to_spawn_count);
    let duration = start.elapsed();
    println!("Spawning cells took {:?}", duration);
}

fn spawn_blinker_cells(world: &mut World, width: u32, height: u32) {
    let start = Instant::now();
    let cells_to_spawn_count = width * height;
    let to_spawn = (0..cells_to_spawn_count).map(|i| {
        let x = i % width;
        let y = i / width;
        let position = Position { x: x as i32, y: y as i32 };
        let state = match (x, y) {
            (1, 0) => State(true),
            (1, 1) => State(true),
            (1, 2) => State(true),
            _ => State(false),
        };
        CellBundle {
            position,
            state,
            ..Default::default()
        }
    });

    world.spawn_batch(to_spawn);
    println!("Spawning {:?} cells", cells_to_spawn_count);
    let duration = start.elapsed();
    println!("Spawning cells took {:?}", duration);
}

fn update_neighbors_system(
    mut query: Query<(&mut Neighbors, &Position)>,
    kd_tree_resource: Res<KdTreeResource>
) {
    let start = Instant::now();
    query.par_iter_mut().for_each(|(mut neighbors, position)| {
        let point = [position.x as f64, position.y as f64];
        let neighbors_within_range = kd_tree_resource.kd_tree
            .within(&point, 2.0, &squared_euclidean)
            .unwrap();
        let count = neighbors_within_range
            .iter()
            .filter(
                |&&(_, pos)|
                    (pos.0 - position.x).abs() <= 1 &&
                    (pos.1 - position.y).abs() <= 1 &&
                    *pos != (position.x, position.y)
            )
            .count() as u8;
        // println!("Updating neighbors for position {:?} with count {:?}", position, count);
        neighbors.0 = count;
    });

    let duration = start.elapsed();
    println!("Updating neighbors took {:?}", duration);
}

fn update_cells_system(
    mut query: Query<(&mut State, &Neighbors)>,
    mut cells_changed: ResMut<CellsChanged>
) {
    let start = Instant::now();
    for (mut state, neighbors) in query.iter_mut() {
        let previous_state = state.0;
        match (state.0, neighbors.0) {
            (true, 2) | (true, 3) => (),
            (false, 3) => {
                state.0 = true;
            }
            _ => {
                state.0 = false;
            }
        }

        if state.0 != previous_state {
            cells_changed.0 = true;
        }
    }
    let duration = start.elapsed();
    println!("Updating cells took {:?}", duration);
}

fn decrease_generation_system(mut generations: ResMut<Generations>) {
    println!("Decreasing generations to {:?}", generations.0);
    if generations.0 > 0 {
        generations.0 -= 1;
    }
}

fn print_all_entities_system(mut query: Query<(Entity, &Position, &State, &Neighbors)>) {
    println!("Printing all entities");
    for (entity, position, state, neighbors) in &mut query {
        println!(
            "Entity {:?} has position {:?}, state {:?}, and neighbors {:?}",
            entity,
            position,
            state,
            neighbors
        );
    }
}

pub fn initialize(width: u32, height: u32, generations: u32) {
    let start = Instant::now();
    let mut world = World::new();
    world.insert_resource(Grid { width, height });
    world.insert_resource(KdTreeResource { kd_tree: KdTree::new(K_DIMENSIONS) });
    world.insert_resource(CellsChanged(true));
    spawn_cells(&mut world, width, height);
    world.insert_resource(Generations { 0: generations });
    let mut schedule = Schedule::default();
    schedule.add_systems((
        (
            rebuild_kd_tree,
            update_neighbors_system,
            update_cells_system,
            rebuild_kd_tree,
            update_neighbors_system,
        ).chain(),
        // print_all_entities_system,
        decrease_generation_system,
    ));

    for _ in 0..generations {
        schedule.run(&mut world);
    }

    let duration = start.elapsed();

    println!("Simulation took {:?}", duration);
}

#[cfg(test)]
mod tests {
    use bevy_ecs::system::RunSystemOnce;

    use super::*;

    #[test]
    fn test_block() {
        let mut world = World::new();
        world.insert_resource(Grid { width: 2, height: 2 });
        world.insert_resource(KdTreeResource { kd_tree: KdTree::new(K_DIMENSIONS) });
        world.insert_resource(CellsChanged(true));
        spawn_block_cells(&mut world, 2, 2);

        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                rebuild_kd_tree,
                update_neighbors_system,
                update_cells_system,
                rebuild_kd_tree,
                update_neighbors_system,
            ).chain()
        );
        schedule.run(&mut world);

        let mut query = world.query::<(&Position, &State, &Neighbors)>();
        let cells = query.iter(&world).collect::<Vec<_>>();
        assert_eq!(cells.len(), 4);
        assert_eq!(*cells[0].1, State(true));
        assert_eq!(*cells[1].1, State(true));
        assert_eq!(*cells[2].1, State(true));
        assert_eq!(*cells[3].1, State(true));
        assert_eq!(*cells[0].2, Neighbors(3));
        assert_eq!(*cells[1].2, Neighbors(3));
        assert_eq!(*cells[2].2, Neighbors(3));
        assert_eq!(*cells[3].2, Neighbors(3));

        schedule.run(&mut world);

        let cells = query.iter(&world).collect::<Vec<_>>();
        assert_eq!(cells.len(), 4);
        assert_eq!(cells.len(), 4);
        assert_eq!(*cells[0].1, State(true));
        assert_eq!(*cells[1].1, State(true));
        assert_eq!(*cells[2].1, State(true));
        assert_eq!(*cells[3].1, State(true));
        assert_eq!(*cells[0].2, Neighbors(3));
        assert_eq!(*cells[1].2, Neighbors(3));
        assert_eq!(*cells[2].2, Neighbors(3));
        assert_eq!(*cells[3].2, Neighbors(3));

        schedule.run(&mut world);

        let cells = query.iter(&world).collect::<Vec<_>>();
        assert_eq!(cells.len(), 4);
        assert_eq!(cells.len(), 4);
        assert_eq!(*cells[0].1, State(true));
        assert_eq!(*cells[1].1, State(true));
        assert_eq!(*cells[2].1, State(true));
        assert_eq!(*cells[3].1, State(true));
        assert_eq!(*cells[0].2, Neighbors(3));
        assert_eq!(*cells[1].2, Neighbors(3));
        assert_eq!(*cells[2].2, Neighbors(3));
        assert_eq!(*cells[3].2, Neighbors(3));
    }

    #[test]
    fn test_beehive() {
        let mut world = World::new();
        world.insert_resource(Grid { width: 6, height: 3 });
        world.insert_resource(KdTreeResource { kd_tree: KdTree::new(K_DIMENSIONS) });
        world.insert_resource(CellsChanged(true));
        spawn_beehive_cells(&mut world, 6, 3);
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                rebuild_kd_tree,
                update_neighbors_system,
                update_cells_system,
                rebuild_kd_tree,
                update_neighbors_system,
            ).chain()
        );
        schedule.run(&mut world);

        let mut query = world.query::<(&Position, &State, &Neighbors)>();
        let cells = query.iter(&world).collect::<Vec<_>>();
        assert_eq!(cells.len(), 18);
        assert_eq!(*cells[0].1, State(false));
        assert_eq!(*cells[1].1, State(false));
        assert_eq!(*cells[2].1, State(true));
        assert_eq!(*cells[3].1, State(true));
        assert_eq!(*cells[4].1, State(false));
        assert_eq!(*cells[5].1, State(false));
        assert_eq!(*cells[6].1, State(false));
        assert_eq!(*cells[7].1, State(true));
        assert_eq!(*cells[8].1, State(false));
        assert_eq!(*cells[9].1, State(false));
        assert_eq!(*cells[10].1, State(true));
        assert_eq!(*cells[11].1, State(false));
        assert_eq!(*cells[12].1, State(false));
        assert_eq!(*cells[13].1, State(false));
        assert_eq!(*cells[14].1, State(true));
        assert_eq!(*cells[15].1, State(true));
        assert_eq!(*cells[16].1, State(false));
        assert_eq!(*cells[17].1, State(false));

        assert_eq!(*cells[0].2, Neighbors(1));
        assert_eq!(*cells[1].2, Neighbors(2));
        assert_eq!(*cells[2].2, Neighbors(2));
        assert_eq!(*cells[3].2, Neighbors(2));
        assert_eq!(*cells[4].2, Neighbors(2));
        assert_eq!(*cells[5].2, Neighbors(1));

        assert_eq!(*cells[6].2, Neighbors(1));
        assert_eq!(*cells[7].2, Neighbors(2));
        assert_eq!(*cells[8].2, Neighbors(5));
        assert_eq!(*cells[9].2, Neighbors(5));
        assert_eq!(*cells[10].2, Neighbors(2));
        assert_eq!(*cells[11].2, Neighbors(1));

        assert_eq!(*cells[12].2, Neighbors(1));
        assert_eq!(*cells[13].2, Neighbors(2));
        assert_eq!(*cells[14].2, Neighbors(2));
        assert_eq!(*cells[15].2, Neighbors(2));
        assert_eq!(*cells[16].2, Neighbors(2));
        assert_eq!(*cells[17].2, Neighbors(1));

        schedule.run(&mut world);

        let cells = query.iter(&world).collect::<Vec<_>>();
        assert_eq!(cells.len(), 18);
        assert_eq!(*cells[0].1, State(false));
        assert_eq!(*cells[1].1, State(false));
        assert_eq!(*cells[2].1, State(true));
        assert_eq!(*cells[3].1, State(true));
        assert_eq!(*cells[4].1, State(false));
        assert_eq!(*cells[5].1, State(false));
        assert_eq!(*cells[6].1, State(false));
        assert_eq!(*cells[7].1, State(true));
        assert_eq!(*cells[8].1, State(false));
        assert_eq!(*cells[9].1, State(false));
        assert_eq!(*cells[10].1, State(true));
        assert_eq!(*cells[11].1, State(false));
        assert_eq!(*cells[12].1, State(false));
        assert_eq!(*cells[13].1, State(false));
        assert_eq!(*cells[14].1, State(true));
        assert_eq!(*cells[15].1, State(true));
        assert_eq!(*cells[16].1, State(false));
        assert_eq!(*cells[17].1, State(false));

        assert_eq!(*cells[0].2, Neighbors(1));
        assert_eq!(*cells[1].2, Neighbors(2));
        assert_eq!(*cells[2].2, Neighbors(2));
        assert_eq!(*cells[3].2, Neighbors(2));
        assert_eq!(*cells[4].2, Neighbors(2));
        assert_eq!(*cells[5].2, Neighbors(1));

        assert_eq!(*cells[6].2, Neighbors(1));
        assert_eq!(*cells[7].2, Neighbors(2));
        assert_eq!(*cells[8].2, Neighbors(5));
        assert_eq!(*cells[9].2, Neighbors(5));
        assert_eq!(*cells[10].2, Neighbors(2));
        assert_eq!(*cells[11].2, Neighbors(1));

        assert_eq!(*cells[12].2, Neighbors(1));
        assert_eq!(*cells[13].2, Neighbors(2));
        assert_eq!(*cells[14].2, Neighbors(2));
        assert_eq!(*cells[15].2, Neighbors(2));
        assert_eq!(*cells[16].2, Neighbors(2));
        assert_eq!(*cells[17].2, Neighbors(1));
    }

    #[test]
    fn test_blinker() {
        let mut world = World::new();
        world.insert_resource(Grid { width: 6, height: 3 });
        world.insert_resource(KdTreeResource { kd_tree: KdTree::new(K_DIMENSIONS) });
        world.insert_resource(CellsChanged(true));
        spawn_blinker_cells(&mut world, 3, 3);
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                rebuild_kd_tree,
                update_neighbors_system,
                update_cells_system,
                rebuild_kd_tree,
                update_neighbors_system,
            ).chain()
        );
        println!("First run");
        schedule.run(&mut world);
        world.run_system_once(print_all_entities_system);

        let mut query = world.query::<(&Position, &State, &Neighbors)>();
        let cells = query.iter(&world).collect::<Vec<_>>();
        assert_eq!(cells.len(), 9);

        assert_eq!(*cells[0].1, State(false));
        assert_eq!(*cells[1].1, State(false));
        assert_eq!(*cells[2].1, State(false));

        assert_eq!(*cells[3].1, State(true));
        assert_eq!(*cells[4].1, State(true));
        assert_eq!(*cells[5].1, State(true));

        assert_eq!(*cells[6].1, State(false));
        assert_eq!(*cells[7].1, State(false));
        assert_eq!(*cells[8].1, State(false));

        assert_eq!(*cells[0].2, Neighbors(2));
        assert_eq!(*cells[1].2, Neighbors(3));
        assert_eq!(*cells[2].2, Neighbors(2));

        assert_eq!(*cells[3].2, Neighbors(1));
        assert_eq!(*cells[4].2, Neighbors(2));
        assert_eq!(*cells[5].2, Neighbors(1));

        assert_eq!(*cells[6].2, Neighbors(2));
        assert_eq!(*cells[7].2, Neighbors(3));
        assert_eq!(*cells[8].2, Neighbors(2));

        println!("Second run");
        schedule.run(&mut world);
        world.run_system_once(print_all_entities_system);

        let cells = query.iter(&world).collect::<Vec<_>>();
        assert_eq!(cells.len(), 9);
        assert_eq!(*cells[0].1, State(false));
        assert_eq!(*cells[1].1, State(true));
        assert_eq!(*cells[2].1, State(false));

        assert_eq!(*cells[3].1, State(false));
        assert_eq!(*cells[4].1, State(true));
        assert_eq!(*cells[5].1, State(false));

        assert_eq!(*cells[6].1, State(false));
        assert_eq!(*cells[7].1, State(true));
        assert_eq!(*cells[8].1, State(false));

        assert_eq!(*cells[0].2, Neighbors(2));
        assert_eq!(*cells[1].2, Neighbors(1));
        assert_eq!(*cells[2].2, Neighbors(2));

        assert_eq!(*cells[3].2, Neighbors(3));
        assert_eq!(*cells[4].2, Neighbors(2));
        assert_eq!(*cells[5].2, Neighbors(3));

        assert_eq!(*cells[6].2, Neighbors(2));
        assert_eq!(*cells[7].2, Neighbors(1));
        assert_eq!(*cells[8].2, Neighbors(2));
    }
}
