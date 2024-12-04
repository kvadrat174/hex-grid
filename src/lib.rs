mod temp_search_grid;
mod temp_node;
mod heap;

use std::collections::{HashMap, HashSet};
use heap::CustomHeap;
use napi_derive::napi;
use serde::Serialize;
use napi::bindgen_prelude::*;
use serde_json::Value;
use temp_node::TempNode;
use temp_search_grid::TempSearchGrid;

#[napi(object)]
pub struct MapHexOptions {
    pub index: u32,
    pub x: i32,
    pub y: i32,
    pub passability: i32,
    pub battleground: bool,
    pub security_index: String,
}
pub enum CustomError {
    NapiError(Error<Status>),
    Panic,
  }

impl AsRef<str> for CustomError {
    fn as_ref(&self) -> &str {
      match self {
        CustomError::Panic => "Panic",
        CustomError::NapiError(e) => e.status.as_ref(),
      }
    }
  }

#[napi]
#[derive(Debug)]
pub enum SecurityIndexType {
    NoBattles,
    Safe,
    NotSafe,
    FreePvP,
  }

#[napi(object)]
pub struct HexBase {
    pub x: i32,                  
    pub y: i32,                     
    pub passability: f64, 
    pub battleground: Option<String>,
    pub security_index: Option<String>,
}

impl HexBase {
    pub fn new(x: i32, y: i32, passability: f64, battleground: Option<String>, security_index: Option<String>) -> Self {
        HexBase {
            x,
            y,
            passability,
            battleground,
            security_index,
        }
    }
}

#[napi(object)]
#[derive(Debug, Serialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[napi(object)]
#[derive(Debug)]
pub struct GridBorder {
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
}
pub struct HexGridOptions<Hex> {
    pub grid_border: GridBorder,
    pub hex_factory: Box<dyn Fn() -> Hex>,
}

#[napi(object)]
#[derive(Debug, Serialize, Clone)]
pub struct Hex {
    pub id: u32,   
    pub x: i32,
    pub y: i32,
    pub passable: bool,
    pub passability: f64,
    pub battleground: bool,
    pub security_index: String,
}

impl Hex {
    /// This is the constructor
    pub fn new(id: u32, x: i32, y: i32, passability: f64, passable: bool, battleground: bool, security_index: String) -> Self {
        Hex { id, x, y, passability, passable, battleground, security_index }
    }
}

#[napi]
pub struct HexGrid {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
    width: i32,
    height: i32,
    hex_id_map: HashMap<i32, usize>,
    template_search_grid: TempSearchGrid,
    odd_incriment: i32,
    hexes: Vec<Hex>
}

#[napi]
impl HexGrid {
    #[napi]
    pub fn new(grid_border: GridBorder, hexes: Vec<HexBase>) -> Result<HexGrid, String> {
        // Calculate width and height based on grid borders
        let min_x = grid_border.min_x;
        let max_x = grid_border.max_x;
        let min_y = grid_border.min_y;
        let max_y = grid_border.max_y;

        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;

        let hexes_len = width * height;

        // Ensure the width and height are valid
        if width < 1 {
            return Err(Error::new(Status::InvalidArg.to_string(), "Grid width cannot be less than 1".to_string()));
        }

        if height < 1 {
            return Err(Error::new(Status::InvalidArg.to_string(), "Grid height cannot be less than 1".to_string()));
        }

        // Determine odd increment based on min_x
        let odd_incriment = min_x & 1;
 
        // Initialize the template search grid
        let mut template_search_grid = TempSearchGrid::new(width.try_into().unwrap(), height.try_into().unwrap(), odd_incriment.try_into().unwrap());

        // Initialize hex storage
        let mut hexes_out = Vec::with_capacity((hexes_len) as usize);
        let mut hex_id_map = HashMap::new();

        // Map HexBase to Hex
        for hex_idx in 0..hexes_len {
            let hex_base = &hexes[hex_idx as usize];
            let hex = Hex {
                id: hex_idx as u32,
                x: hex_base.x,
                y: hex_base.y,
                passable: hex_base.passability > 0.0,
                passability: hex_base.passability,
                battleground: hex_base.battleground.is_some(),
                security_index: match &hex_base.security_index {
                    Some(security_index) => security_index.to_string(),
                    None => String::from("not_safe"),
                },
            };
 
            // Insert hex into the grid and update the search grid
            hex_id_map.insert(hex.id.try_into().expect("Must be defined"), hex_idx as usize);
            hexes_out.push(hex.clone());

            // Update passability in the search grid
            let point = (&hex.x - min_x, &hex.y - min_y);

            if !hex.passable {
                let _ = template_search_grid.set_node_passable(point.0.try_into().unwrap(), point.1.try_into().unwrap(), false);
                let _ = template_search_grid
                    .recheck_node_passable(point.0.try_into().unwrap(), point.1.try_into().unwrap())
                    .map_err(|e| format!("Failed to set node passable: {}", e));
            }

            let _ = template_search_grid
                .set_node_passability(point.0.try_into().unwrap(), point.1.try_into().unwrap(), hex.passability)
                .map_err(|e| format!("Failed to set node passability: {}", e));
        }

        let mut hex_grid = HexGrid {
            min_x,
            max_x,
            min_y,
            max_y,
            width,
            height,
            odd_incriment,
            template_search_grid,
            hex_id_map,
            hexes: hexes_out,
        };

        // Cache neighbors after all nodes are updated
        let _ = hex_grid.template_search_grid
            .compute_neighbor_nodes_cache()
            .map_err(|e| format!("Failed to compute neighbor nodes cache: {}", e));

        // Return the new HexGrid
        Ok(hex_grid)
    }

    pub fn transform_hex_point_to_node_point(&self, hex: &Hex) -> (i32, i32) {
        (hex.x - self.min_x, hex.y - self.min_y)
    }

    pub fn transform_node_point_to_hex_point(&self, x: usize, y: usize) -> (i32, i32) {
        ((x as i32) + self.min_x, (y as i32) + self.min_y)
    }

    #[napi(getter)]
    pub fn get_hexes(&self) -> Result<String> {
        serde_json::to_string(&self.hexes).map_err(|err| {
            napi::Error::from_reason(format!(
                "Failed to serialize hexes to JSON: {}",
                err
            ))
        })
    }

    #[napi(getter)]
    pub fn getwidth(&self) -> Result<i32> {
        Ok(self.width.into())
    }

    fn odd_increment(&self) -> i32 {
        self.odd_incriment
    }

    #[napi(getter)]
    pub fn get_height(&self) -> Result<i32> {
        Ok(self.height.into())
    }

    #[napi]
    pub fn is_inside_radius(&self, center: Point, target: Point, radius: f64) -> bool {
        let distance = self.calculate_distance_between_hexes(center, target);
        distance <= radius
    }

    #[napi]
    pub fn calculate_distance_between_hexes(&self, point_a: Point, point_b: Point) -> f64 {
        Self::heuristic_odd_q(point_a.x.try_into().unwrap(), point_a.y.try_into().unwrap(), point_b.x, point_b.y)
    }

    #[napi]
    pub fn find_hex_by_position(&self, x: i32, y: i32) -> Option<Hex> {
        // Check if the position is within the boundaries
        if self.is_within_boundaries(x, y) {
            // Calculate the index based on the position
            let index = (y - self.min_y) * self.width + (x - self.min_x);
    
            // Check if the index is within bounds of the hexes vector
            if index >= 0 && index < self.hexes.len() as i32 {
                // Return a clone of the Hex at the calculated index
                return Some(self.hexes[index as usize].clone());
            }
        }
        None
    }

    // Helper function to check if (x, y) is within grid boundaries
    fn is_within_boundaries(&self, x: i32, y: i32) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    pub fn get_position_by_id(&self, hex_id: usize) -> Option<Hex> {
        // Access the hex by index (assuming _hexes is a Vec<Hex>)
        let hex = self.hexes.get(hex_id); // get returns Option<&Hex>
        
        // If the hex exists, return a Point, otherwise return None
        match hex {
            Some(hex) => Some(hex.clone()),
            None => None,  // Return None if the hex does not exist
        }
    }

    #[napi]
    pub fn get_neighbours_by_id(&self, hex_id: u32) -> Result<Vec<Hex>, String> {
        // Get the position of the hex by its ID
        let position = self.get_position_by_id(hex_id.try_into().unwrap()).unwrap();
        
        // Retrieve the starting node from the search grid using the position
        let hex_point = self.transform_hex_point_to_node_point(&position);
        let start_node = self.template_search_grid.get_node_at_point(((hex_point.0 as usize), (hex_point.1 as usize)));
        // Get the neighbors of the node, assuming get_neighbors_passable_nodes_from_cache exists
        let neighbors = self.template_search_grid.get_neighbors_passable_nodes_from_cache(start_node.x, start_node.y).unwrap();

        // Convert the nodes into corresponding Hexes and return
        let hexes: Vec<Hex> = neighbors
            .into_iter()
            .map(|node| self.get_hex_by_node_position(node).unwrap())
            .collect();

        Ok(hexes)
    }
    #[napi]
    pub fn is_hex_on_border_of_impasable_area(&self, point: Point) -> bool {
        
        let node_point =  (point.x - self.min_x, point.y - self.min_y);
        let node = self.template_search_grid.get_node_at_point(((node_point.0 as usize), (node_point.1 as usize)));

       if node.passable {
            return false;
       }

        self.template_search_grid.is_node_on_border_of_impassable_area(node.x, node.y)
    }

    pub fn is_hex_in_border_by_id(&self, id: i32) -> bool {
        // Check if the ID is within the valid range (0 to width * height)
        id >= 0 && id < ((self.height * self.width) as u32).try_into().unwrap()
    }

    #[napi]
    pub fn get_hexes_within_range(&self, center: Point, range: i32) -> Vec<Hex> {
        let center_hex = match self.find_hex_by_position(center.x, center.y) {
            Some(hex) => hex,
            None => return Vec::new(), // Return empty if center hex isn't found
        };

        let center_hex_id = center_hex.id;
        let mut result = Vec::new();

        for i in -range..=range {
            for l in -range..=range {
                let hex_id = center_hex_id as i32 + (i * self.width) + l;

                // Check if the hex is within the border of the grid
                if self.is_hex_in_border_by_id(hex_id) {
                    if let Some(hex) = self.get_hex_by_id(hex_id.try_into().unwrap()) {
                        result.push(hex.clone()); // Add the hex to the result
                    }
                }
            }
        }

        result
    }
    #[napi]
    pub fn get_border_passable_neighbors(&self, point: Point) -> Vec<Hex> {
        
        let node_point =  (point.x - self.min_x, point.y - self.min_y);
        let node = self.template_search_grid.get_node_at_point(((node_point.0 as usize), (node_point.1 as usize)));
        let neighbours = self.template_search_grid.get_border_passable_neighbors(node.x, node.y).unwrap_or(Vec::new());
        
        let hexes = neighbours.into_iter()
        .map(|(x, y)| {
            // Transform node point back into hex point
            let hex_point = ((x as i32) + self.min_x, (y as i32) + self.min_y);
            let hex: Hex = self.find_hex_by_position(hex_point.0, hex_point.1).unwrap();
            hex
        })
        .collect();
        hexes
    }

    #[napi(ts_return_type="Record<number, Hex>[]")]
    pub fn get_difference_between_areas_with_neigbor_centers(
        &self,
        old_center: Point,
        new_center: Point,
        range: i32,
    ) -> Vec<Value> {
        let center_hex = self.find_hex_by_position(new_center.x, new_center.y).unwrap();
        let center_hex_id = center_hex.id as i32;
    
        let mut added: HashMap<i32, Hex> = HashMap::new();
        let mut deleted: HashMap<i32, Hex> = HashMap::new();
        let mut result = Vec::new();
        let mut index: i32;
    
        // Left movements (x-1)
        if new_center.x == old_center.x - 1 {
            if new_center.y == old_center.y {
                // Pure left
                for i in -range..=range {
                    // Left edge (added)
                    index = center_hex_id + i * self.width - range;
                    if self.is_hex_in_border_by_id(index) {
                        added.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Right edge (deleted)
                    index = center_hex_id + i * self.width + range + 1;
                    if self.is_hex_in_border_by_id(index) {
                        deleted.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                }
            } else if new_center.y == old_center.y - 1 {
                // Left-up diagonal
                for i in -range..=range {
                    // Top edge (added)
                    index = center_hex_id - range * self.width + i;
                    if self.is_hex_in_border_by_id(index) {
                        added.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Left edge (added)
                    index = center_hex_id - i * self.width - range;
                    if self.is_hex_in_border_by_id(index) {
                        added.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Bottom edge (deleted)
                    index = center_hex_id + (range + 1) * self.width + i + 1;
                    if self.is_hex_in_border_by_id(index) {
                        deleted.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Right edge (deleted)
                    index = center_hex_id + (i + 1) * self.width + range + 1;
                    if self.is_hex_in_border_by_id(index) {
                        deleted.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                }
            } else if new_center.y == old_center.y + 1 {
                // Left-down diagonal
                for i in -range..=range {
                    // Bottom edge (added)
                    index = center_hex_id + range * self.width + i;
                    if self.is_hex_in_border_by_id(index) {
                        added.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Left edge (added)
                    index = center_hex_id + i * self.width - range;
                    if self.is_hex_in_border_by_id(index) {
                        added.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Top edge (deleted)
                    index = center_hex_id - (range + 1) * self.width + i + 1;
                    if self.is_hex_in_border_by_id(index) {
                        deleted.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Right edge (deleted)
                    index = center_hex_id + (i - 1) * self.width + range + 1;
                    if self.is_hex_in_border_by_id(index) {
                        deleted.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                }
            }
        }
        // Right movements (x+1)
        else if new_center.x == old_center.x + 1 {
            if new_center.y == old_center.y {
                // Pure right
                for i in -range..=range {
                    // Right edge (added)
                    index = center_hex_id + i * self.width + range;
                    if self.is_hex_in_border_by_id(index) {
                        added.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Left edge (deleted)
                    index = center_hex_id + i * self.width - range - 1;
                    if self.is_hex_in_border_by_id(index) {
                        deleted.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                }
            } else if new_center.y == old_center.y - 1 {
                // Right-up diagonal
                for i in -range..=range {
                    // Top edge (added)
                    index = center_hex_id - range * self.width + i;
                    if self.is_hex_in_border_by_id(index) {
                        added.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Right edge (added)
                    index = center_hex_id - i * self.width + range;
                    if self.is_hex_in_border_by_id(index) {
                        added.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Bottom edge (deleted)
                    index = center_hex_id + (range + 1) * self.width + i - 1;
                    if self.is_hex_in_border_by_id(index) {
                        deleted.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Left edge (deleted)
                    index = center_hex_id + (i + 1) * self.width - range - 1;
                    if self.is_hex_in_border_by_id(index) {
                        deleted.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                }
            } else if new_center.y == old_center.y + 1 {
                // Right-down diagonal
                for i in -range..=range {
                    // Bottom edge (added)
                    index = center_hex_id + range * self.width + i;
                    if self.is_hex_in_border_by_id(index) {
                        added.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Right edge (added)
                    index = center_hex_id + i * self.width + range;
                    if self.is_hex_in_border_by_id(index) {
                        added.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Top edge (deleted)
                    index = center_hex_id - (range + 1) * self.width + i - 1;
                    if self.is_hex_in_border_by_id(index) {
                        deleted.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Left edge (deleted)
                    index = center_hex_id + (i - 1) * self.width - range - 1;
                    if self.is_hex_in_border_by_id(index) {
                        deleted.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                }
            }
        }
        // Vertical movements (same x)
        else if new_center.x == old_center.x {
            if new_center.y == old_center.y - 1 {
                // Pure up
                for i in -range..=range {
                    // Top edge (added)
                    index = center_hex_id - range * self.width + i;
                    if self.is_hex_in_border_by_id(index) {
                        added.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Bottom edge (deleted)
                    index = center_hex_id + (range + 1) * self.width + i;
                    if self.is_hex_in_border_by_id(index) {
                        deleted.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                }
            } else if new_center.y == old_center.y + 1 {
                // Pure down
                for i in -range..=range {
                    // Bottom edge (added)
                    index = center_hex_id + range * self.width + i;
                    if self.is_hex_in_border_by_id(index) {
                        added.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                    // Top edge (deleted)
                    index = center_hex_id - (range + 1) * self.width + i;
                    if self.is_hex_in_border_by_id(index) {
                        deleted.insert(index, self.get_hex_by_id(index.try_into().unwrap()).unwrap().clone());
                    }
                }
            }
        }
    
        let added_val = serde_json::to_value(added).unwrap();
        result.push(added_val);
    
        let deleted_val = serde_json::to_value(deleted).unwrap();
        result.push(deleted_val);
        result
    }
    #[napi]
    pub fn build_path_to_impassable_border_hex(
        &mut self,
        start_id: u32,
        target_id: u32,
    ) -> Result<Vec<Point>, String> {
        // Retrieve the start and target Hexes by ID
        let start_hex = self.get_hex_by_id(start_id).unwrap();
        let target_hex = self.get_hex_by_id(target_id).unwrap();

        // Ensure the start hex is passable
        if !start_hex.passable {
            return Err(Error::new(Status::InvalidArg.to_string(), "StartHex is not passable".to_string()));
        }

        // Reset the search grid for a new pathfinding operation
        self.template_search_grid.reset();

        // Transform hex points to node points
        let start_node_point = self.transform_hex_point_to_node_point(&start_hex);
        let target_node_point = self.transform_hex_point_to_node_point(&target_hex);

        let target_node = self
        .template_search_grid
        .get_node_at_point(((target_node_point.0 as usize), (target_node_point.1 as usize)));
        let terminal_nodes: Vec<TempNode> = vec![*target_node];

        // Calculate the path from the start to the target hex
        let path_matrix_positions = self.calculate_path_by_algorithm(
            start_node_point,
            target_node_point,
            &terminal_nodes,
        )?;

        let point_path = path_matrix_positions
        .into_iter()
        .map(|(x, y)| {
            // Transform node point back into hex point
            let hex_point = self.transform_node_point_to_hex_point(
                x.try_into().unwrap(),
                y.try_into().unwrap(),
            );
            Point { x: hex_point.0, y: hex_point.1 }
        })
        .collect();
        Ok(point_path)
    }

    #[napi]
    pub fn build_path_towards_impassable_hex(
        &mut self,
        start_id: u32,
        target_id: u32,
    ) -> Result<Vec<Point>, String> {
        let start_hex = self.get_hex_by_id(start_id).unwrap();
        let target_hex = self.get_hex_by_id(target_id).unwrap();
        if !start_hex.passable {
            return Err(Error::new(Status::InvalidArg.to_string(), "StartHex is not passable".to_string()));
        }
        self.template_search_grid.reset();

        let start_node_point = self.transform_hex_point_to_node_point(&start_hex);
        let target_node_point = self.transform_hex_point_to_node_point(&target_hex);

        let terminal_node_points = self.template_search_grid
        .get_border_passable_neighbors(target_node_point.0.try_into().unwrap(), target_node_point.1.try_into().unwrap()).unwrap();
        
        let terminal_nodes: Vec<TempNode> = terminal_node_points
        .into_iter()
        .map(|(x, y)| {
            *self.template_search_grid.get_node_at_point((x, y))
        })
        .collect();

        let path_matrix_positions = self.calculate_path_by_algorithm(
            start_node_point,
            target_node_point,
            &terminal_nodes,
        )?;
        
        // println!("{:?}", path_matrix_positions);
        let point_path = path_matrix_positions
        .into_iter()
        .map(|(x, y)| {
            // Transform node point back into hex point
            let hex_point = self.transform_node_point_to_hex_point(
                x.try_into().unwrap(),
                y.try_into().unwrap(),
            );
            Point { x: hex_point.0, y: hex_point.1 }
        })
        .collect();
    Ok(point_path)
    }
    #[napi]
    pub fn build_path_to_passable_hex(
        &mut self,
        start_id: u32,
        target_id: u32,
    ) -> Result<Vec<Point>, String> {
        // Retrieve the start and target Hexes by ID
        let start_hex = self.get_hex_by_id(start_id).unwrap();
        let target_hex = self.get_hex_by_id(target_id).unwrap();

        // Check if the start hex is passable
        if !start_hex.passable {
            return Err(Error::new(Status::InvalidArg.to_string(), "StartHex is not passable".to_string()));
        }
    
        // Reset the search grid for a new pathfinding operation
        self.template_search_grid.reset();
        
        let start_node_point = self.transform_hex_point_to_node_point(&start_hex);
        let target_node_point = self.transform_hex_point_to_node_point(&target_hex);

        let target_node = self
            .template_search_grid
            .get_node_at_point(((target_node_point.0 as usize), (target_node_point.1 as usize)));
        let terminal_nodes: Vec<TempNode> = vec![*target_node];
    
        // Calculate the path from the start to the target hex
        let path_matrix_positions = self.calculate_path_by_algorithm(
            start_node_point,
            target_node_point,
            &terminal_nodes,
        )?;

        let point_path: Vec<Point> = path_matrix_positions
        .into_iter()
        .map(|(x, y)| {
            let hex_point = self.transform_node_point_to_hex_point(
                x.try_into().unwrap(),
                y.try_into().unwrap(),
            );
            Point { x: hex_point.0, y: hex_point.1 }
        })
        .collect();

    Ok(point_path)

    }
    

    pub fn get_hex_by_id(&self, id: u32) -> Option<Hex> {
        let hex_id = self.hex_id_map.get(&(id as i32)).unwrap();
        Some(self.hexes[*hex_id].clone())
    }

    pub fn get_hex_by_node_position(&self, node_point: TempNode) -> Result<Hex, String> {
        // Calculate the index in the _hexes vector
        let index = node_point.y * (self.width as usize) + node_point.x;

        // Check if the index is within bounds of the _hexes vector
        if index < self.hexes.len() {
            let hex = &self.hexes[index];

            // Check if the hex is within the boundaries
            if self.is_within_boundaries(hex.x, hex.y) {
                Ok(hex.clone())  // Return a copy of the Hex (or clone if needed)
            } else {
                return Err(Error::new(Status::InvalidArg.to_string(), "Hex is out of boundaries".to_string()));
            }
        } else {
            return Err(Error::new(Status::InvalidArg.to_string(), "Invalid node position: out of bounds".to_string()));
        }
    }


    pub fn calculate_path_by_algorithm(
        &mut self,
        start_point: (i32, i32),
        end_point: (i32, i32),
        terminal_nodes: &[TempNode],
    ) -> Result<Vec<(usize, usize)>, String> {
        // let mut ng: f64;

        let mut open_list = CustomHeap::new(self.hexes.len());
        let terminal_node_set: HashSet<(usize, usize)> = terminal_nodes.iter().map(|n| (n.x, n.y)).collect();
        // Select the heuristic function

        let heuristic: fn(usize, usize, i32, i32) -> f64 = if self.odd_increment() != 0 {
            Self::heuristic_even_q
        } else {
            Self::heuristic_odd_q
        }; 

        let start_x: usize = start_point.0.try_into().unwrap();
        let start_y: usize = start_point.1.try_into().unwrap();
        let end_x = end_point.0;
        let end_y = end_point.1;

        let _ = self.template_search_grid.update_node(start_x, start_y, |n| {
            n.f = 0.0;
            n.g = 0.0;
            n.opened = Some(true);
        });
        open_list.push((0.0, start_x, start_y));

        while !open_list.is_empty() {
            let (x, y) = open_list.pop().unwrap();
            let current_g = *self.template_search_grid.get_node_g_at_point((x, y));
        
            let _ = self.template_search_grid.set_node_closed(x, y, true);

            if terminal_node_set.contains(&(x, y)) {
                return Ok(self.backtrace(self.template_search_grid.get_node_at_point((x, y))));
            }
        
            // Get neighbors of the current node
            let neighbors = self
            .template_search_grid
            .get_neighbors_passable_nodes_from_cache(x, y)
            .unwrap();
            // println!("----------{}, {}-----------", x,y);
            for neighbor in neighbors {

                if neighbor.closed.unwrap_or(false) {
                    continue; 
                }
        
                // Calculate `g` score (cost to get to this neighbor)
                let ng = current_g + (1.0 / neighbor.passability);
                let neighbour_f;
                let mut neighbour_h= neighbor.h;

                if !neighbor.opened.unwrap_or(false) {

                    if neighbor.h.is_none() {
                        neighbour_h = Some(heuristic(neighbor.x, neighbor.y, end_x, end_y));

                        let _ = self.template_search_grid.set_node_h(neighbor.x, neighbor.y, neighbour_h.unwrap());
                    }
                    neighbour_f = ng + neighbour_h.unwrap();

                    let _ = self.template_search_grid.update_node(neighbor.x, neighbor.y, |n| {
                        n.f = neighbour_f;
                        n.g = ng;
                        n.parent = Some((x, y));
                        n.opened = Some(true);
                    });
                    open_list.push((neighbour_f, neighbor.x, neighbor.y));
                    
                } else if ng < neighbor.g {

                    neighbour_f = ng + neighbour_h.unwrap();
                let _ = self.template_search_grid.update_node(neighbor.x, neighbor.y, |n| {
                    n.f = neighbour_f;
                    n.g = ng;
                    n.parent = Some((x, y));
                });
                    open_list.update((neighbour_f, neighbor.x, neighbor.y));
                }
            }
        }
        Err(Error::new(Status::InvalidArg.to_string(), format!(
            "Path not found from [{}, {}] to [{}, {}]",
            start_point.0, start_point.1, end_point.0, end_point.1
        )))
    }

    
    fn backtrace(&self, node: &TempNode) -> Vec<(usize, usize)> {
        let mut path = vec![(node.x, node.y)];
        // println!("{:?}", node);
        let mut current = node.parent;
        while let Some(n) = current {
            let node = self.template_search_grid.get_node_at_point(n);
            // println!("{:?}", node);
            path.push((n.0, n.1));
            current = node.parent;
        }

        path.reverse();

        path
    }

    fn heuristic_odd_q(x: usize, y: usize, end_x: i32, end_y: i32) -> f64 {
        // Преобразуем все переменные в f64 для согласованности
        let x = x as f64;
        let mut y = y as f64;
        let end_x = end_x as f64;
        let mut end_y = end_y as f64;
    
        // Сдвиг координат y и end_y
        y -= (x - (x % 2.0)) / 2.0;
        end_y -= (end_x - (end_x % 2.0)) / 2.0;

        // Вычисление эвристики
        let h = (x - end_x).abs()
        + (x + y - end_x - end_y).abs()
        + (y - end_y).abs();
         h / 2.0
    }
    
    fn heuristic_even_q(x: usize, y: usize, end_x: i32, end_y: i32) -> f64 {
        let yy = y - ((x + (x & 1)) / 2);
        let end_yy = end_y - ((end_x + (end_x & 1)) / 2);
    
        // Heuristic formula
        (f64::abs(x as f64 - end_x as f64) + f64::abs(x as f64 + yy as f64 - end_x as f64 - end_yy as f64) + f64::abs(yy as f64 - end_yy as f64)) / 2.0
    }
}

