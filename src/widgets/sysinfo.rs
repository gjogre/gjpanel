use nvml_wrapper::Nvml;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Padding},
};
use sysinfo::{Components, System};

use crate::logger::Logger;

use super::GJWidget;

pub struct SysInfoWidget {
    logger: &'static Logger,
    system: System,
    component_info: String,
    mem_percent: u64,
    swap_percent: u64,
    kernel_version: String,
    cpu_load: u64,
    cpu_temp: u64,
    gpu_mem_usage: u64,
    gpu_temp: u64,
    gpu_util: u64,
    nvml: Option<Nvml>,
}

impl SysInfoWidget {
    pub fn new(logger: &'static Logger) -> Self {
        let nvml_option = Nvml::init();
        let nvml = match nvml_option {
            Ok(nvml) => Some(nvml),
            Err(err) => {
                logger.error(&format!("Failed to initialize NVML: {}", err));
                None
            }
        };
        Self {
            logger,
            system: System::new_all(),
            component_info: String::new(),
            mem_percent: 0,
            swap_percent: 0,
            kernel_version: String::new(),
            cpu_load: 0,
            cpu_temp: 0,
            nvml,
            gpu_mem_usage: 0,
            gpu_temp: 0,
            gpu_util: 0,
        }
    }

    fn set_cpu_usage(&mut self) {
        self.component_info = String::new();

        let mut cpu_usage = 0.0;
        for cpu in self.system.cpus() {
            cpu_usage += cpu.cpu_usage();
        }
        self.cpu_load = (cpu_usage / self.system.cpus().len() as f32) as u64;
        let components = Components::new_with_refreshed_list();
        if components.is_empty() {
            self.logger.error("No components detected.\n");
        } else {
            for component in &components {
                if component.label() == "Tctl" {
                    self.cpu_temp = component.temperature().unwrap() as u64;
                }
                self.component_info.push_str(&format!(
                    "{}: {:.1?}°C (max: {:.1?}°C / crit: {:.1}°C)\n",
                    component.label(),
                    component.temperature(),
                    component.max(),
                    component.critical().unwrap_or(f32::NAN) // Critical temp might not be available
                ));
            }
        }
    }
    fn set_gpu_usage(&mut self) {
        if let Some(nvml) = &self.nvml {
            match nvml.device_by_index(0) {
                Ok(device) => {
                    match device.memory_info() {
                        Ok(memory_info) => {
                            self.gpu_mem_usage =
                                (memory_info.used as f64 / memory_info.total as f64 * 100.0) as u64;
                        }
                        Err(err) => {
                            self.logger
                                .error(&format!("Failed to get GPU memory info: {}", err));
                        }
                    }
                    match device
                        .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                    {
                        Ok(temperature) => {
                            self.gpu_temp = temperature as u64;
                        }
                        Err(err) => {
                            self.logger
                                .error(&format!("Failed to get GPU temperature: {}", err));
                        }
                    }
                    match device.utilization_rates() {
                        Ok(utilization_rates) => {
                            self.gpu_util = utilization_rates.gpu as u64;
                        }
                        Err(err) => {
                            self.logger
                                .error(&format!("Failed to get GPU utilization rates: {}", err));
                        }
                    }
                }

                Err(err) => {
                    self.logger
                        .error(&format!("Failed to get GPU device: {}", err));
                }
            }
        }
    }

    fn set_memory_usage(&mut self) {
        self.mem_percent =
            (self.system.used_memory() as f64 / self.system.total_memory() as f64 * 100.0) as u64;
        self.swap_percent =
            (self.system.used_swap() as f64 / self.system.total_swap() as f64 * 100.0) as u64;
    }
}

impl GJWidget for SysInfoWidget {
    fn render(&self, f: &mut Frame, area: Rect) {
        let cpu_bar_group = BarGroup::default().label("CPU 󰍛".into()).bars(&[
            Bar::default()
                .value(self.cpu_load)
                .text_value(self.cpu_load.to_string() + " %"),
            Bar::default()
                .value(self.cpu_temp)
                .text_value(self.cpu_temp.to_string() + " °C"),
        ]);

        let mem_bar_group = BarGroup::default().label("MEM ".into()).bars(&[
            Bar::default()
                .value(self.mem_percent)
                .text_value(self.mem_percent.to_string() + " %"),
            Bar::default()
                .value(self.swap_percent)
                .text_value(self.swap_percent.to_string() + " S%"),
        ]);
        let gpu_bar_group = BarGroup::default().label("GPU 󱡶".into()).bars(&[
            Bar::default()
                .value(self.gpu_util)
                .text_value(self.gpu_util.to_string() + " %"),
            Bar::default()
                .value(self.gpu_temp)
                .text_value(self.gpu_temp.to_string() + " °C"),
            Bar::default()
                .value(self.gpu_mem_usage)
                .text_value(self.gpu_mem_usage.to_string() + " MB%"),
        ]);

        let bar_chart = BarChart::default()
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .title(self.kernel_version.clone())
                    .title_style(Style::new().dark_gray().on_black())
                    .title_alignment(Alignment::Left)
                    .padding(Padding::top(2)),
            )
            .direction(ratatui::layout::Direction::Horizontal)
            .bar_width(1)
            .bar_gap(0)
            .group_gap(2)
            .bar_style(Style::new().dark_gray().on_black())
            .value_style(Style::new().black().on_dark_gray())
            .label_style(Style::new().dark_gray())
            .data(cpu_bar_group)
            .data(gpu_bar_group)
            .data(mem_bar_group)
            .max(100);

        // Render the Paragraph widget in the given area
        f.render_widget(bar_chart, area);
        //f.render_widget(deb, area);
    }
    fn poll(&mut self) {
        _ = &self.system.refresh_all();

        self.set_cpu_usage();
        self.set_memory_usage();
        self.set_gpu_usage();
        self.kernel_version = System::kernel_version().unwrap();

        //let cpu_usage = self.system.global_cpu_info().cpu_usage();
    }
}
