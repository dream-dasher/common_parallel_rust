use eframe::egui;
use std::collections::VecDeque;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskType {
        Chop,  // Equivalent to ADD/SUB (fast, 1 cycle)
        Grind, // Equivalent to MUL (medium speed, 3 cycles)
        Sauce, // Equivalent to FPU ops (4 cycles)
        Batch, // Equivalent to SIMD (1 cycle but processes multiple)
}

#[derive(Debug, Clone)]
struct Task {
        task_type: TaskType,
        duration: u64,
}

struct ExecutionUnit {
        unit_type: TaskType,
        busy_cycles: u64,
}

impl ExecutionUnit {
        fn new(unit_type: TaskType) -> Self {
                ExecutionUnit { unit_type, busy_cycles: 0 }
        }

        fn process(&mut self, task: &Task) {
                if self.unit_type == task.task_type {
                        self.busy_cycles = task.duration;
                }
        }

        fn tick(&mut self) {
                if self.busy_cycles > 0 {
                        self.busy_cycles -= 1;
                }
        }

        fn is_available(&self) -> bool {
                self.busy_cycles == 0
        }
}

struct Kitchen {
        queue: VecDeque<Task>,
        chefs: Vec<ExecutionUnit>,
        cycle: u64,
}

impl Kitchen {
        fn new() -> Self {
                Kitchen {
                        queue: VecDeque::new(),
                        chefs: vec![
                                ExecutionUnit::new(TaskType::Chop),
                                ExecutionUnit::new(TaskType::Grind),
                                ExecutionUnit::new(TaskType::Sauce),
                                ExecutionUnit::new(TaskType::Batch),
                        ],
                        cycle: 0,
                }
        }

        fn add_task(&mut self, task: Task) {
                self.queue.push_back(task);
        }

        fn tick(&mut self) {
                self.cycle += 1;
                for chef in self.chefs.iter_mut() {
                        chef.tick();
                }

                let mut assigned_tasks = Vec::new();
                for (i, task) in self.queue.iter().enumerate() {
                        for chef in self.chefs.iter_mut() {
                                if chef.is_available() && chef.unit_type == task.task_type {
                                        chef.process(task);
                                        assigned_tasks.push(i);
                                        break;
                                }
                        }
                }

                for &i in assigned_tasks.iter().rev() {
                        self.queue.remove(i);
                }
        }
}

struct KitchenApp {
        kitchen: Kitchen,
}

impl KitchenApp {
        fn new() -> Self {
                KitchenApp { kitchen: Kitchen::new() }
        }
}

impl eframe::App for KitchenApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
                self.kitchen.tick();
                egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading("Kitchen Execution Simulator");
                        ui.label(format!("Cycle: {}", self.kitchen.cycle));

                        ui.separator();
                        ui.label("Queue:");
                        for task in &self.kitchen.queue {
                                ui.label(format!("{:?} ({} cycles)", task.task_type, task.duration));
                        }

                        ui.separator();
                        ui.label("Chefs:");
                        for chef in &self.kitchen.chefs {
                                let status = if chef.is_available() { "Idle" } else { "Busy" };
                                let color = if chef.is_available() { egui::Color32::GREEN } else { egui::Color32::RED };
                                ui.colored_label(color, format!("{:?}: {}", chef.unit_type, status));
                        }

                        ui.separator();
                        ui.label("Add Tasks:");
                        if ui.button("🍴 Add Chop Task").clicked() {
                                self.kitchen.add_task(Task { task_type: TaskType::Chop, duration: 1 });
                        }
                        if ui.button("🔪 Add Grind Task").clicked() {
                                self.kitchen.add_task(Task { task_type: TaskType::Grind, duration: 3 });
                        }
                        if ui.button("🥣 Add Sauce Task").clicked() {
                                self.kitchen.add_task(Task { task_type: TaskType::Sauce, duration: 4 });
                        }
                        if ui.button("🍱 Add Batch Task").clicked() {
                                self.kitchen.add_task(Task { task_type: TaskType::Batch, duration: 1 });
                        }
                });
                sleep(Duration::from_millis(200));
        }
}

fn main() {
        let native_options = eframe::NativeOptions::default();
        eframe::run_native("Kitchen Simulator", native_options, Box::new(|_cc| Ok(Box::new(KitchenApp::new()))))
                .unwrap();
}
