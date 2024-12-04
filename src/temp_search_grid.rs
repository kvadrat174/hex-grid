use std::collections::{HashMap, HashSet};
use crate::temp_node::TempNode;

pub struct TempSearchGrid {
    width: usize,
    height: usize,
    odd_increment: usize,
    nodes: Vec<Vec<TempNode>>,
    neighbor_node_cache: HashMap<(usize, usize), Vec<(usize, usize)>>,
    neighbor_passable_nodes_cache: HashMap<(usize, usize), Vec<(usize, usize)>>,
}

impl TempSearchGrid {
    pub fn new(width: usize, height: usize, odd_increment: usize) -> Self {
        let mut grid = Self {
            width,
            height,
            odd_increment,
            nodes: Vec::new(),
            neighbor_node_cache: HashMap::new(),
            neighbor_passable_nodes_cache: HashMap::new(),
        };
    
        // Now build the nodes using the instance method
        grid.nodes = grid.build_nodes();
        grid
    }

    pub fn build_nodes(&mut self) -> Vec<Vec<TempNode>> {
        let mut nodes = Vec::with_capacity(self.height);
    
        for i in 0..self.height {
            let mut row = Vec::with_capacity(self.width);
            for j in 0..self.width {
                let node = TempNode::new(j, i, true, 1.0);
                row.push(node);
            }
            nodes.push(row); 
        }
    
        nodes
    }


    pub fn compute_neighbor_nodes_cache(&mut self) -> Result<(), String> {
        for node_row in &self.nodes {
            for node in node_row {
                // Get the neighbors for the current node
                let neighbors = self.get_neighbor_nodes(node);
                let coords: Vec<(usize, usize)> = neighbors
                .iter()
                .map(|node| (node.x as usize, node.y as usize))
                .collect();
                self.neighbor_node_cache.insert((node.x.try_into().unwrap(), node.y.try_into().unwrap()), coords);

                // Get the passable neighbors for the current node
                let neighbors_passable_nodes = self.get_neighbors_passable_nodes(node);
                let neighbors_coords: Vec<(usize, usize)> = neighbors_passable_nodes
                .iter()
                .map(|node| (node.x as usize, node.y as usize))
                .collect();
                self.neighbor_passable_nodes_cache.insert((node.x.try_into().expect("Can not insert cache"), node.y.try_into().unwrap()), neighbors_coords);

            }
        }
        Ok(())
    }

    pub fn recheck_node_passable(&mut self, x: usize, y: usize) -> Result<(), String> {
        let node  = self.get_node_at_point((x, y));

        let neighbors = self.get_neighbor_nodes(node);

        let neighbor_passable_nodes = self.get_neighbors_passable_nodes(node)
        .iter()
        .map(|node| (node.x as usize, node.y as usize))
        .collect::<Vec<(usize, usize)>>();
        self.neighbor_passable_nodes_cache.insert((node.x, node.y), neighbor_passable_nodes);
    
        for neighbor in neighbors {
            let neighbor_neighbors = self.get_neighbors_passable_nodes(&neighbor)
            .iter()
            .filter(|&&TempNode { passable, .. }| *passable)
            .map(|&TempNode { x, y, .. }| (*x as usize, *y as usize))
            .collect::<Vec<(usize, usize)>>();
        self.neighbor_passable_nodes_cache.insert((neighbor.x, neighbor.y), neighbor_neighbors);
        }
        Ok(())
    }

    pub fn set_node_passable(&mut self, x: usize, y: usize, passable: bool) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(y).and_then(|row| row.get_mut(x)) {
            node.set_passable(passable);
            Ok(())
        } else {
            Err(format!("Invalid node coordinates: ({}, {})", x, y))
        }
    }

    pub fn set_node_passability(&mut self, x: usize, y: usize, passability: f64) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(y).and_then(|row| row.get_mut(x)) {
            node.set_passability(passability);
            Ok(())
        } else {
            Err(format!("Invalid node coordinates: ({}, {})", x, y))
        }
    }
    pub fn set_node_closed(&mut self, x: usize, y: usize, v: bool) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(y).and_then(|row| row.get_mut(x)) {
            node.set_closed(v);
            Ok(())
        } else {
            Err(format!("Invalid node coordinates: ({}, {})", x, y))
        }
    }
    pub fn set_node_opened(&mut self, x: usize, y: usize, v: bool) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(y).and_then(|row| row.get_mut(x)) {
            node.set_opened(v);
            Ok(())
        } else {
            Err(format!("Invalid node coordinates: ({}, {})", x, y))
        }
    }
    pub fn set_node_g(&mut self, x: usize, y: usize, v: f64) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(y).and_then(|row| row.get_mut(x)) {
            node.set_g(v);
            Ok(())
        } else {
            Err(format!("Invalid node coordinates: ({}, {})", x, y))
        }
    }
    pub fn set_node_h(&mut self, x: usize, y: usize, v: f64) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(y).and_then(|row| row.get_mut(x)) {
            node.set_h(v);
            Ok(())
        } else {
            Err(format!("Invalid node coordinates: ({}, {})", x, y))
        }
    }
    pub fn set_node_parent(&mut self, x: usize, y: usize, v: (usize, usize)) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(y).and_then(|row| row.get_mut(x)) {
            node.set_parent(v);
            Ok(())
        } else {
            Err(format!("Invalid node coordinates: ({}, {})", x, y))
        }
    }
    pub fn update_node<F>(&mut self, x: usize, y: usize, mut f: F)
    where
        F: FnMut(&mut TempNode),
    {
        if let Some(row) = self.nodes.get_mut(y) {
            if let Some(node) = row.get_mut(x) {
                f(node);
            }
        }
    }
    pub fn get_neighbor_nodes_from_cache(&self, node: &TempNode) -> Result<Vec<TempNode>, String> {
        // Look up the neighbors from the cache using the node's coordinates
        let neighbors_coords = self.neighbor_node_cache
            .get(&(node.x.try_into().unwrap(), node.y.try_into().unwrap())) // Fetch neighbors' coordinates using the (x, y) tuple as the key
            .ok_or_else(|| format!("Neighbor cache is empty for node at ({}, {})", node.x, node.y))?;

        // Map coordinates to actual TempNode instances
        let neighbors: Vec<TempNode> = neighbors_coords
            .iter()
            .map(|&(nx, ny)| *self.get_node_at_point((nx.try_into().unwrap(), ny.try_into().unwrap())))
            .collect();

        Ok(neighbors)
    }

    pub fn get_neighbors_passable_nodes_from_cache(&self, x: usize, y: usize) -> Result<Vec<TempNode>, String> {
        // Get the coordinates of passable neighbors from the cache
        let neighbors_coords = self.neighbor_passable_nodes_cache
            .get(&(x, y))
            .ok_or_else(|| format!("Neighbor passable nodes cache is empty for ({}, {})", x, y))?;

        let neighbors: Vec<TempNode> = neighbors_coords
            .iter()
            .map(|&(nx, ny)| *self.get_node_at_point((nx, ny))) // Convert coordinates into TempNode
            .collect();
    
        Ok(neighbors)
    }

    pub fn get_neighbors_passable_node_points_from_cache(&self, x: usize, y: usize) -> &Vec<(usize, usize)> {
        // Get the coordinates of passable neighbors from the cache
        let neighbors_coords: &Vec<(usize, usize)> = self.neighbor_passable_nodes_cache
            .get(&(x, y))
            .ok_or_else(|| format!("Neighbor passable nodes cache is empty for ({}, {})", x, y)).unwrap();
        neighbors_coords
    }

    pub fn is_node_on_border_of_impassable_area(&self, x: usize, y: usize) -> bool {
        let neighbors = self.get_neighbors_passable_nodes_from_cache(x, y).unwrap_or(Vec::new());
        if neighbors.len() > 0 {
            return true;
        }
        false
    }

    pub fn get_border_passable_neighbors(&self, target_x: usize, target_y: usize) -> Result<Vec<(usize, usize)>, String> {
        let mut open_neighbors = HashSet::new();
        let mut closed_neighbors = HashSet::new();
        let mut done_nodes = HashSet::new();

        closed_neighbors.insert((target_x, target_y));

        while !closed_neighbors.is_empty() {
            let current = closed_neighbors.iter().cloned().next().unwrap();
            closed_neighbors.remove(&current);

            let neighbors = self.neighbor_node_cache
                .get(&current)
                .ok_or_else(|| format!("Cache empty for {:?}", current))?;

            for &neighbor in neighbors {
                if self.nodes[neighbor.1][neighbor.0].passable {
                    open_neighbors.insert(neighbor);
                } else if !done_nodes.contains(&neighbor) {
                    closed_neighbors.insert(neighbor);
                }
            }
            done_nodes.insert(current);
        }

        Ok(open_neighbors.into_iter().collect())
    }

    pub fn get_nodes_within_range(&self, center_x: usize, center_y: usize, range: usize) -> Result<Vec<(usize, usize)>, String> {
        let mut area = HashSet::new();
        let mut nodes_of_interest = HashSet::new();
        let mut done_nodes = HashSet::new();

        area.insert((center_x, center_y));
        nodes_of_interest.insert((center_x, center_y));

        while !nodes_of_interest.is_empty() {
            let current = nodes_of_interest.iter().cloned().next().unwrap();
            nodes_of_interest.remove(&current);

            let neighbors = self.neighbor_node_cache
                .get(&current)
                .ok_or_else(|| format!("Cache empty for {:?}", current))?;

            for &neighbor in neighbors {
                if self.is_node_in_range(center_x, center_y, neighbor.0, neighbor.1, range) {
                    area.insert(neighbor);
                    if !done_nodes.contains(&neighbor) {
                        nodes_of_interest.insert(neighbor);
                    }
                }
            }
            done_nodes.insert(current);
        }

        Ok(area.into_iter().collect())
    }

    pub fn is_node_in_range(&self, x0: usize, y0: usize, x1: usize, y1: usize, range: usize) -> bool {
        (x1 as isize - x0 as isize).abs() as usize <= range
            && (y1 as isize - y0 as isize).abs() as usize <= range
    }

    fn is_node_passable(&self, x: usize, y: usize) -> bool {
        if self.is_node_inside(x, y) {
            let node  = self.get_node_at_point((x.try_into().unwrap(), y.try_into().unwrap()));
            node.passable
        } else {
            false
        }
    }

    pub fn is_node_inside(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
   }

    pub fn get_neighbors_passable_nodes(&self, node: &TempNode) -> Vec<&TempNode> {
        let mut neighbors = Vec::new();
        let x = node.x as usize;
        let y = node.y as usize;

        if self.is_node_passable(x, y - 1) {
            neighbors.push(self.get_node_at_point((node.x, node.y - 1)));
        }
        if self.is_node_passable(x, y + 1) {
            neighbors.push(self.get_node_at_point((node.x, node.y + 1)));
        }
        if (x + self.odd_increment) % 2 == 0 {
            if self.is_node_passable(x + 1, y - 1) {
                neighbors.push(self.get_node_at_point((node.x + 1, node.y - 1)));
            }
            if self.is_node_passable(x - 1, y - 1) {
                neighbors.push(self.get_node_at_point((node.x - 1, node.y - 1)));
            }
            if self.is_node_passable(x + 1, y) {
                neighbors.push(self.get_node_at_point((node.x + 1, node.y)));
            }
            if self.is_node_passable(x - 1, y) {
                neighbors.push(self.get_node_at_point((node.x - 1, node.y)));
            }
        } else {
            if self.is_node_passable(x + 1, y) {
                neighbors.push(self.get_node_at_point((node.x + 1, node.y)));
            }
            if self.is_node_passable(x + 1, y + 1) {
                neighbors.push(self.get_node_at_point((node.x + 1, node.y + 1)));
            }
            if self.is_node_passable(x - 1, y + 1) {
                neighbors.push(self.get_node_at_point((node.x - 1, node.y + 1)));
            }
            if self.is_node_passable(x - 1, y) {
                neighbors.push(self.get_node_at_point((node.x -1, node.y)));
            }
        }

        neighbors
    }
    

    pub fn get_neighbor_nodes(&self, node: &TempNode) -> Vec<TempNode> {
        let mut neighbors = Vec::new();
        let x = node.x as usize;
        let y = node.y as usize;

        if self.is_node_inside(x, y - 1) {
            neighbors.push(*self.get_node_at_point((node.x, node.y - 1)));
        }
        if self.is_node_inside(x, y + 1) {
            neighbors.push(*self.get_node_at_point((node.x, node.y + 1)));
        }
        if (x + self.odd_increment) % 2 == 0 {
            if self.is_node_inside(x + 1, y - 1) {
                neighbors.push(*self.get_node_at_point((node.x + 1, node.y - 1)));
            }
            if self.is_node_inside(x - 1, y - 1) {
                neighbors.push(*self.get_node_at_point((node.x - 1, node.y - 1)));
            }
            if self.is_node_inside(x + 1, y) {
                neighbors.push(*self.get_node_at_point((node.x + 1, node.y)));
            }
            if self.is_node_inside(x - 1, y) {
                neighbors.push(*self.get_node_at_point((node.x - 1, node.y)));
            }
        } else {
            if self.is_node_inside(x + 1, y) {
                neighbors.push(*self.get_node_at_point((node.x + 1, node.y)));
            }
            if self.is_node_inside(x + 1, y + 1) {
                neighbors.push(*self.get_node_at_point((node.x + 1, node.y + 1)));
            }
            if self.is_node_inside(x - 1, y + 1) {
                neighbors.push(*self.get_node_at_point((node.x - 1, node.y + 1)));
            }
            if self.is_node_inside(x - 1, y) {
                neighbors.push(*self.get_node_at_point((node.x - 1, node.y)));
            }
        }
        neighbors
    }

    pub fn get_node_at_point(&self, point: (usize, usize)) -> &TempNode {
        let (x, y) = (point.0, point.1);
        &self.nodes[y][x]
    }

    pub fn get_node_g_at_point(&self, point: (usize, usize)) -> &f64 {
        let (x, y) = (point.0, point.1);
        &self.nodes[y][x].g
    }

    // Method to reset all nodes in the grid
    pub fn reset(&mut self) {
        // Iterate over every node in the grid and reset it
        for row in &mut self.nodes {
            for node in row {
                node.reset(); // Call the reset method on each TempNode
            }
        }
    }
}
