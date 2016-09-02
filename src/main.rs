extern crate libc;
extern crate xcb;
extern crate nalgebra;
extern crate rand;
extern crate time;
extern crate unicode_normalization;
extern crate thread_scoped;
#[macro_use] extern crate log;
extern crate ansi_term;
extern crate freetype_sys;
extern crate glob;
extern crate epoll;
extern crate socket;
#[macro_use] mod vkffi;
mod render_vk;
mod prelude;

mod constants;
use constants::*;
mod traits;
mod vertex_formats;
use vertex_formats::*;
mod structures;
mod logical_resources;
mod utils;
use nalgebra::*;
use rand::distributions::*;
mod evdev;
use evdev::*;
mod udev;
use udev::*;

use vkffi::*;

use std::collections::LinkedList;
use std::cell::RefCell;
use std::sync::{Mutex, Arc};

use prelude::traits::*;

struct Enemy<'a>
{
	datastore_ref: &'a RefCell<logical_resources::EnemyDatastore<'a>>,
	block_index: u32, left: f32, living_secs: f32
}
impl <'a> Enemy<'a>
{
	pub fn new(datastore: &'a RefCell<logical_resources::EnemyDatastore<'a>>, init_left: f32) -> Option<Self>
	{
		let mut datastore_ref = datastore.borrow_mut();
		datastore_ref.allocate_block().map(move |index|
		{
			datastore_ref.update_instance_data(index,
				UnitQuaternion::new(Vector3::new(0.0f32, 0.0f32, 0.0f32)).quaternion(), UnitQuaternion::new(Vector3::new(0.0f32, 0.0f32, 0.0f32)).quaternion(),
				&Vector4::new(init_left, 0.0f32, 0.0f32, 0.0f32));
			Enemy
			{
				datastore_ref: datastore, block_index: index, left: init_left, living_secs: 0.0f32
			}
		})
	}
	pub fn update(&mut self, delta_time: f32) -> bool
	{
		let current_y = if self.living_secs < 0.875f32
		{
			15.0f32 * (1.0f32 - (1.0f32 - self.living_secs / 0.875f32).powi(2)) - 3.0f32
		}
		else
		{
			15.0f32 + (self.living_secs - 0.875f32) * 2.5f32 - 3.0f32
		};
		self.datastore_ref.borrow_mut().update_instance_data(self.block_index,
			UnitQuaternion::new(Vector3::new(-1.0f32, 0.0f32, 0.75f32).normalize() * (260.0f32 * self.living_secs).to_radians()).quaternion(),
			UnitQuaternion::new(Vector3::new(1.0f32, -1.0f32, 0.5f32).normalize() * (-260.0f32 * self.living_secs + 13.0f32).to_radians()).quaternion(),
			&Vector4::new(self.left, current_y, 0.0f32, 0.0f32));
		self.living_secs += delta_time;

		current_y >= 50.0f32
	}
	pub fn die(self)
	{
		self.datastore_ref.borrow_mut().free_block(self.block_index);
	}
}

struct Player<'a>
{
	uniform_memory: &'a mut structures::CVector4, instance_memory: &'a mut [structures::CVector4; 2],
	living_secs: f32
}
impl <'a> Player<'a>
{
	fn new(uniform_ref: &'a mut structures::CVector4, instance_ref: &'a mut [structures::CVector4; 2]) -> Self
	{
		let u_quaternion = UnitQuaternion::new(Vector3::new(0.0f32, 0.0f32, 0.0f32));
		let quaternion_ref = u_quaternion.quaternion();

		instance_ref[0] = [quaternion_ref.i, quaternion_ref.j, quaternion_ref.k, quaternion_ref.w];
		instance_ref[1] = [quaternion_ref.i, quaternion_ref.j, quaternion_ref.k, quaternion_ref.w];
		*uniform_ref = [0.0f32, 38.0f32, 0.0f32, 0.0f32];

		Player
		{
			uniform_memory: uniform_ref, instance_memory: instance_ref,
			living_secs: 0.0f32
		}
	}
	fn update(&mut self, frame_delta: f32, input: &InputSystem<LogicalInputTypes>)
	{
		let u_quaternions = [
			UnitQuaternion::new(Vector3::new(-1.0f32, 0.0f32, 0.75f32).normalize() * (260.0f32 * self.living_secs as f32).to_radians()),
			UnitQuaternion::new(Vector3::new(1.0f32, -1.0f32, 0.5f32).normalize() * (-260.0f32 * self.living_secs as f32 + 13.0f32).to_radians())
		];
		let mut quaternions = u_quaternions.iter().map(|x| x.quaternion()).map(|q| [q.i, q.j, q.k, q.w]);
		self.living_secs += frame_delta;

		self.uniform_memory[0] =
			(self.uniform_memory[0] + input[LogicalInputTypes::Horizontal] * 40.0f32 * frame_delta).max(-33.0f32).min(33.0f32);
		self.uniform_memory[1] =
			(self.uniform_memory[1] + input[LogicalInputTypes::Vertical] * 40.0f32 * frame_delta).max(1.5f32).min(45.0f32);

		self.instance_memory[0] = quaternions.next().unwrap();
		self.instance_memory[1] = quaternions.next().unwrap();
	}
}

use std::collections::HashMap;
type AsyncExclusiveHashMap<K, V> = Arc<Mutex<HashMap<K, V>>>;
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum LogicalInputTypes
{
	Horizontal, Vertical, Shoot, Slowdown, Overdrive
}
pub enum InputType
{
	Key(KeyEvents), Axis(AbsoluteAxisEvents), KeyAsAxis(KeyEvents, KeyEvents)
}
pub struct InputDevice
{
	dev: EventDevice,
	key_states: HashMap<KeyEvents, bool>,
	axis_prev_values: HashMap<AbsoluteAxisEvents, f32>
}
impl InputDevice
{
	pub fn new(node_path: &str) -> Result<InputDevice, prelude::EngineError>
	{
		EventDevice::new(node_path).map(|ev| InputDevice
		{
			dev: ev,
			key_states: HashMap::new(),
			axis_prev_values: HashMap::new()
		})
	}
	pub fn update(&mut self, aggregate_key_states: &mut HashMap<KeyEvents, u32>, aggregate_axis_states: &mut HashMap<AbsoluteAxisEvents, f32>)
	{
		if let Ok(ev) = self.dev.wait_event()
		{
			match ev
			{
				DeviceEvent::Syn(_, _) => (),
				DeviceEvent::Key(_, k, p) => match p
				{
					PressedState::Released =>
					{
						*self.key_states.entry(k).or_insert(false) = false;
						*aggregate_key_states.entry(k).or_insert(1) -= 1;
					},
					PressedState::Pressed =>
					{
						*self.key_states.entry(k).or_insert(true) = true;
						*aggregate_key_states.entry(k).or_insert(0) += 1;
					},
					PressedState::Repeating => ()
				},
				DeviceEvent::Absolute(_, x, v) =>
				{
					let old_value = *self.axis_prev_values.entry(x).or_insert(0.0f32);
					*aggregate_axis_states.entry(x).or_insert(0.0f32) -= old_value;
					*aggregate_axis_states.entry(x).or_insert(0.0f32) += v;
					*self.axis_prev_values.entry(x).or_insert(0.0f32) = v;
				},
				_ => ()
			}
		}
	}
	pub fn unplug(self, aggregate_key_states: &mut HashMap<KeyEvents, u32>, aggregate_axis_states: &mut HashMap<AbsoluteAxisEvents, f32>)
	{
		for (k, v) in self.key_states
		{
			if v { *aggregate_key_states.entry(k).or_insert(1) -= 1; }
		}
		for (x, v) in self.axis_prev_values
		{
			*aggregate_axis_states.entry(x).or_insert(v) -= v;
		}
	}
}
impl std::os::unix::io::AsRawFd for InputDevice
{
	fn as_raw_fd(&self) -> std::os::unix::io::RawFd { self.dev.as_raw_fd() }
}
pub struct InputSystem<InputNames: PartialEq + Eq + std::hash::Hash + Copy + Clone + std::fmt::Debug>
{
	keymap: std::collections::HashMap<InputNames, Vec<InputType>>,
	aggregate_key_states: AsyncExclusiveHashMap<KeyEvents, u32>,
	aggregate_axis_states: AsyncExclusiveHashMap<AbsoluteAxisEvents, f32>,
	input_states: std::collections::HashMap<InputNames, f32>
}
use std::os::unix::io::AsRawFd;
impl <InputNames: PartialEq + Eq + std::hash::Hash + Copy + Clone + std::fmt::Debug> InputSystem<InputNames>
{
	pub fn new() -> Result<Self, prelude::EngineError>
	{
		let aks = Arc::new(Mutex::new(HashMap::new()));
		let aas = Arc::new(Mutex::new(HashMap::new()));
		let aks_thread = aks.clone();
		let aas_thread = aas.clone();

		try!(std::thread::Builder::new().name("Input Thread".into()).spawn(move ||
		{
			let mut input_devices = HashMap::new();
			info!(target: "Prelude::Input", "Starting udev...");
			let udev = UserspaceDeviceManager::new().unwrap();
			
			info!(target: "Prelude::Input", "Listing Event Devices...");
			let enumerator = udev.new_enumerator().unwrap().filter_match_subsystem("input");
			for dev in enumerator.get_devices()
			{
				let device_name = dev.name().and_then(|x| x.to_str().ok());
				let is_event_device = device_name.and_then(|dev_name| dev_name.split('/').last())
					.map(|final_name| final_name.starts_with("event")).unwrap_or(false);
				if is_event_device
				{
					// event_device
					let dev_name = device_name.unwrap();
					debug!(target: "Prelude::Input", "Event Device: {:?}", dev_name);
					let device = udev.new_device_from_syspath(&dev_name);
					// search device name ascending parent
					let mut par_dev_opt = device.parent();
					let mut device_name = None;
					while let Some(pardev) = par_dev_opt
					{
						if let Some(dn) = pardev.property_value("NAME")
						{
							device_name = Some(dn.to_str().unwrap().to_owned());
							break;
						}
						par_dev_opt = pardev.parent();
					}
					let device_name = device_name.unwrap_or(String::from("Unknown Device"));
					let device_node = device.device_node().unwrap().to_str().unwrap().to_owned();
					let node_number = device_node["/dev/input/event".len()..].parse::<u32>().unwrap();
					info!(target: "Prelude::Input", "Initializing for Input: {} [{}]", device_name, device_node);
					debug!(target: "Prelude::Input", "-- Initialized: {}", device.is_initialized());
					let joystick_device = device.property_value("ID_INPUT_JOYSTICK").and_then(|f| f.to_str().ok()).map(|n| n == "1").unwrap_or(false);
					let keyboard_device = device.property_value("ID_INPUT_KEYBOARD").and_then(|f| f.to_str().ok()).map(|n| n == "1").unwrap_or(false);
					if joystick_device
					{
						info!(target: "Prelude::Input", "-- Identified as Joystick");
						input_devices.insert(node_number, InputDevice::new(&device_node).unwrap());
					}
					else if keyboard_device
					{
						info!(target: "Prelude::Input", "-- Identified as Keyboard");
						input_devices.insert(node_number, InputDevice::new(&device_node).unwrap());
					}
					/*for props in device.properties()
					{
						info!(target: "Prelude::Input", "-- Property: {:?} = {:?}", props.name(), props.value());
					}*/
				}
			}

			let udev_monitor = udev.new_monitor().unwrap().add_filter_subsystem("input").enable_receiving();
			let mut polling = epoll::EpollInstance::new().expect("Unable to create polling object");
			for (n, d) in input_devices.iter()
			{
				polling.add_interest(epoll::Interest::new(d.as_raw_fd(), epoll::EPOLLIN, *n as u64)).unwrap();
			}
			polling.add_interest(epoll::Interest::new(udev_monitor.as_raw_fd(), epoll::EPOLLIN, std::u64::MAX)).unwrap();
			while let Ok(events) = polling.wait(-1, input_devices.len())
			{
				for event in events
				{
					if event.data() == std::u64::MAX
					{
						// from udev_monitor
						if let Ok(dev) = udev_monitor.receive_device()
						{
							let is_event_device = dev.device_node().and_then(|d| d.to_str().ok()).map(|d| d.starts_with("/dev/input/event")).unwrap_or(false);
							if is_event_device
							{
								// event device
								let device_action = dev.action().unwrap().to_str().unwrap().to_owned();
								debug!(target: "Prelude::Input", "hotplug notified from udev");
								debug!(target: "Prelude::Input", "-- Action: {}", device_action);
								let device_node = dev.device_node().unwrap().to_str().unwrap().to_owned();
								let node_number = device_node["/dev/input/event".len()..].parse::<u32>().unwrap();
								// search device name ascending parent
								let mut par_dev_opt = dev.parent();
								let mut device_name = None;
								while let Some(pardev) = par_dev_opt
								{
									if let Some(dn) = pardev.property_value("NAME")
									{
										device_name = Some(dn.to_str().unwrap().to_owned());
										break;
									}
									par_dev_opt = pardev.parent();
								}
								let device_name = device_name.unwrap_or(String::from("Unknown Device"));
								debug!(target: "Prelude::Input", "-- Name = {}", device_name);
								debug!(target: "Prelude::Input", "-- Node = {:?}", device_node);
								debug!(target: "Prelude::Input", "-- Node Number = {}", node_number);
								debug!(target: "Prelude::Input", "-- Initialized: {}", dev.is_initialized());

								match dev.action().and_then(|cs| cs.to_str().ok())
								{
									Some("remove") => if let Some(removed_device) = input_devices.remove(&node_number)
									{
										info!(target: "Prelude::Input", "Removed Device {}", device_name);
										polling.del_interest(&epoll::Interest::new(removed_device.as_raw_fd(), epoll::EPOLLIN, node_number as u64)).unwrap();
										removed_device.unplug(&mut aks_thread.lock().unwrap(), &mut aas_thread.lock().unwrap());
									},
									Some("add") =>
									{
										let joystick_device = dev.property_value("ID_INPUT_JOYSTICK").and_then(|f| f.to_str().ok()).map(|n| n == "1").unwrap_or(false);
										let keyboard_device = dev.property_value("ID_INPUT_KEYBOARD").and_then(|f| f.to_str().ok()).map(|n| n == "1").unwrap_or(false);
										if joystick_device
										{
											info!(target: "Prelude::Input", "Added Device {} as Joystick", device_name);
											let idev = InputDevice::new(&device_node).unwrap();
											polling.add_interest(epoll::Interest::new(idev.as_raw_fd(), epoll::EPOLLIN, node_number as u64)).unwrap();
											input_devices.insert(node_number, idev);
										}
										else if keyboard_device
										{
											info!(target: "Prelude::Input", "Added Device {} as Keyboard", device_name);
											let idev = InputDevice::new(&device_node).unwrap();
											polling.add_interest(epoll::Interest::new(idev.as_raw_fd(), epoll::EPOLLIN, node_number as u64)).unwrap();
											input_devices.insert(node_number, idev);
										}
									},
									_ => ()
								}
							}
						}
						else { warn!(target: "Prelude::Input", "Failed to receive device from udev monitor"); }
					}
					else
					{
						// from input devices
						match input_devices.get_mut(&(event.data() as u32))
						{
							Some(input_device) => input_device.update(&mut aks_thread.lock().unwrap(), &mut aas_thread.lock().unwrap()),
							None =>
							{
								warn!(target: "Prelude::Input", "Input Device is not found?");
							}
						}
					}
				}
			}
		}));

		Ok(InputSystem
		{
			keymap: std::collections::HashMap::new(),
			aggregate_key_states: aks, aggregate_axis_states: aas,
			input_states: std::collections::HashMap::new()
		})
	}
	pub fn add_input(mut self, to: InputNames, from: InputType) -> Self
	{
		self.keymap.entry(to).or_insert(Vec::new()).push(from);
		self.input_states.insert(to, 0.0f32);
		self
	}
	pub fn update(&mut self)
	{
		let (mut key_states, mut axis_states) = (self.aggregate_key_states.lock().unwrap(), self.aggregate_axis_states.lock().unwrap());
		for (t, v) in &self.keymap
		{
			let mut total_value = 0.0f32;
			for f in v
			{
				total_value += match f
				{
					&InputType::Axis(x) => *axis_states.entry(x).or_insert(0.0f32),
					&InputType::Key(k) => if *key_states.entry(k).or_insert(0) > 0 { 1.0f32 } else { 0.0f32 },
					&InputType::KeyAsAxis(n, p) =>
						(if *key_states.entry(p).or_insert(0) > 0 { 1.0f32 } else { 0.0f32 }) -
						(if *key_states.entry(n).or_insert(0) > 0 { 1.0f32 } else { 0.0f32 })
				};
			}
			*self.input_states.entry(*t).or_insert(total_value) = total_value.max(-1.0f32).min(1.0f32);
		}
	}
}
impl <InputNames: PartialEq + Eq + std::hash::Hash + Copy + Clone + std::fmt::Debug> std::ops::Index<InputNames> for InputSystem<InputNames>
{
	type Output = f32;
	fn index(&self, name: InputNames) -> &f32
	{
		static DEFAULT_F32: f32 = 0.0f32;
		self.input_states.get(&name).unwrap_or(&DEFAULT_F32)
	}
}

fn main() { if let Err(e) = app_main() { prelude::crash(e); } }
fn app_main() -> Result<(), prelude::EngineError>
{
	utils::memory_management_test();

	let engine = try!{
		prelude::Engine::new("HardGrad->Extent", VK_MAKE_VERSION!(0, 0, 1))
			.map(|e| e.with_assets_in(std::env::current_dir().unwrap()))
	};
	let main_frame = try!(engine.create_render_window(VkExtent2D(640, 480), "HardGrad -> Extent"));
	let VkExtent2D(frame_width, frame_height) = main_frame.get_extent();
	let execute_next_signal = try!(engine.create_fence());

	let rp_attachment_descs =
	[
		prelude::AttachmentDesc
		{
			format: main_frame.get_format(), clear_on_load: Some(true), preserve_stored_value: true,
			initial_layout: VkImageLayout::ColorAttachmentOptimal, final_layout: VkImageLayout::PresentSrcKHR,
			.. Default::default()
		}
	];
	let render_passes = [prelude::PassDesc::single_fragment_output(0)];
	let rp_framebuffer_form = try!(engine.create_render_pass(&rp_attachment_descs, &render_passes, &[]));
	let framebuffers = try!(main_frame.get_back_images().iter()
		.map(|x| engine.create_framebuffer(&rp_framebuffer_form, &[&x.view], VkExtent3D(frame_width, frame_height, 1)))
		.collect::<Result<Vec<_>, _>>());

	// Resources //
	let application_buffer_prealloc = engine.buffer_preallocate(&[
		(std::mem::size_of::<structures::VertexMemoryForWireRender>(), prelude::BufferDataType::Vertex),
		(std::mem::size_of::<structures::IndexMemory>(), prelude::BufferDataType::Index),
		(std::mem::size_of::<structures::InstanceMemory>(), prelude::BufferDataType::Vertex),
		(std::mem::size_of::<structures::UniformMemory>(), prelude::BufferDataType::Uniform)
	]);
	let (application_data, appdata_stage) = try!(engine.create_double_buffer(&application_buffer_prealloc));

	// setup initial data //
	try!(appdata_stage.map().map(|mapped|
	{
		let vertices = mapped.map_mut::<structures::VertexMemoryForWireRender>(application_buffer_prealloc.offset(0));
		let indices = mapped.map_mut::<structures::IndexMemory>(application_buffer_prealloc.offset(1));
		vertices.unit_plane_source_vts = [
			Position(-1.0f32, -1.0f32, 0.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32, 0.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32, 0.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32, 0.0f32, 1.0f32)
		];
		vertices.player_cube_vts = [
			Position(-1.0f32, -1.0f32, -1.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32, -1.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32, -1.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32, -1.0f32, 1.0f32),
			Position(-1.0f32, -1.0f32,  1.0f32, 1.0f32),
			Position( 1.0f32, -1.0f32,  1.0f32, 1.0f32),
			Position( 1.0f32,  1.0f32,  1.0f32, 1.0f32),
			Position(-1.0f32,  1.0f32,  1.0f32, 1.0f32)
		];
		indices.player_cube_ids = [
			0, 1, 1, 2, 2, 3, 3, 0,
			4, 5, 5, 6, 6, 7, 7, 4,
			0, 4, 1, 5, 2, 6, 3, 7
		];
		let uniforms = mapped.map_mut::<structures::UniformMemory>(application_buffer_prealloc.offset(3));
		logical_resources::projection_matrixes::setup_parameters(uniforms, main_frame.get_extent());
	}));

	// Descriptor Set //
	let dslayout_u1 = try!(engine.create_descriptor_set_layout(&[
		prelude::Descriptor::Uniform(1, vec![prelude::ShaderStage::Vertex, prelude::ShaderStage::Geometry])
	]));
	let all_descriptor_sets = try!(engine.preallocate_all_descriptor_sets(&[&dslayout_u1]));
	engine.update_descriptors(&[
		prelude::DescriptorSetWriteInfo::UniformBuffer(all_descriptor_sets[0], 0, vec![
			prelude::BufferInfo(&application_data, application_buffer_prealloc.offset(3) .. application_buffer_prealloc.total_size() as usize)
		])
	]);

	// Shading Structures //
	let raw_output_vert = try!(engine.create_vertex_shader_from_asset("shaders.RawOutput", "main", &[
		prelude::VertexBinding::PerVertex(std::mem::size_of::<vertex_formats::Position>() as u32),
		prelude::VertexBinding::PerInstance(std::mem::size_of::<u32>() as u32)
	], &[prelude::VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0), prelude::VertexAttribute(1, VkFormat::R32_UINT, 0)]));
	let player_rotor_vert = try!(engine.create_vertex_shader_from_asset("shaders.PlayerRotor", "main", &[
		prelude::VertexBinding::PerVertex(std::mem::size_of::<vertex_formats::Position>() as u32),
		prelude::VertexBinding::PerInstance(std::mem::size_of::<structures::CVector4>() as u32)
	], &[prelude::VertexAttribute(0, VkFormat::R32G32B32A32_SFLOAT, 0), prelude::VertexAttribute(1, VkFormat::R32G32B32A32_SFLOAT, 0)]));
	let backline_duplicator = try!(engine.create_geometry_shader_from_asset("shaders.BackLineDuplicator", "main"));
	let enemy_duplicator = try!(engine.create_geometry_shader_from_asset("shaders.EnemyDuplicator", "main"));
	let through_color_frag = try!(engine.create_fragment_shader_from_asset("shaders.ThroughColor", "main"));

	let swapchain_viewport = VkViewport(0.0f32, 0.0f32, frame_width as f32, frame_height as f32, 0.0f32, 1.0f32);
	let wire_render_layout = try!(engine.create_pipeline_layout(&[&dslayout_u1], &[prelude::PushConstantDesc(VK_SHADER_STAGE_VERTEX_BIT, 0 .. 16)]));
	let background_render_state = prelude::GraphicsPipelineBuilder::new(&wire_render_layout, &rp_framebuffer_form, 0)
		.vertex_shader(&raw_output_vert).geometry_shader(&backline_duplicator).fragment_shader(&through_color_frag)
		.primitive_topology(prelude::PrimitiveTopology::LineList(true))
		.viewport_scissors(&[prelude::ViewportWithScissorRect::default_scissor(swapchain_viewport)])
		.blend_state(&[prelude::AttachmentBlendState::PremultipliedAlphaBlend]);
	let enemy_render_state = prelude::GraphicsPipelineBuilder::inherit(&background_render_state)
	 	.geometry_shader(&enemy_duplicator)
		.blend_state(&[prelude::AttachmentBlendState::Disabled]);
	let player_render_state = prelude::GraphicsPipelineBuilder::new(&wire_render_layout, &rp_framebuffer_form, 0)
		.vertex_shader(&player_rotor_vert).fragment_shader(&through_color_frag)
		.primitive_topology(prelude::PrimitiveTopology::LineList(false))
		.viewport_scissors(&[prelude::ViewportWithScissorRect::default_scissor(swapchain_viewport)])
		.blend_state(&[prelude::AttachmentBlendState::Disabled]);
	let pipeline_states = try!(engine.create_graphics_pipelines(&[&background_render_state, &enemy_render_state, &player_render_state]));
	let ref background_render = pipeline_states[0];
	let ref enemy_render = pipeline_states[1];
	let ref player_render = pipeline_states[2];

	// Initial Data Transmission, Layouting for Swapchain Backbuffer Images //
	try!(engine.allocate_transient_transfer_command_buffers(1).and_then(|setup_commands|
	{
		let buffer_memory_barriers = [
			prelude::BufferMemoryBarrier::hold_ownership(&appdata_stage, 0 .. application_buffer_prealloc.total_size(),
				0, VK_ACCESS_TRANSFER_READ_BIT),
			prelude::BufferMemoryBarrier::hold_ownership(&application_data, 0 .. application_buffer_prealloc.total_size(),
				0, VK_ACCESS_TRANSFER_WRITE_BIT)
		];
		let buffer_memory_barriers_ret = [
			prelude::BufferMemoryBarrier::hold_ownership(&appdata_stage, 0 .. application_buffer_prealloc.total_size(),
				VK_ACCESS_TRANSFER_READ_BIT, VK_ACCESS_MEMORY_READ_BIT),
			prelude::BufferMemoryBarrier::hold_ownership(&application_data, 0 .. application_buffer_prealloc.total_size(),
				VK_ACCESS_TRANSFER_WRITE_BIT, VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT | VK_ACCESS_INDEX_READ_BIT | VK_ACCESS_UNIFORM_READ_BIT)
		];
		let image_memory_barriers = main_frame.get_back_images().iter()
			.map(|x| prelude::ImageMemoryBarrier::hold_ownership(*x, prelude::ImageSubresourceRange::base_color(),
			0, VK_ACCESS_MEMORY_READ_BIT, VkImageLayout::Undefined, VkImageLayout::PresentSrcKHR)).collect::<Vec<_>>();

		try!(setup_commands.begin(0).and_then(|recorder|
			recorder.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, false,
				&[], &buffer_memory_barriers, &image_memory_barriers)
			.copy_buffer(&appdata_stage, &application_data, &[prelude::BufferCopyRegion(0, 0, application_buffer_prealloc.total_size() as usize)])
			.pipeline_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT, false, &[], &buffer_memory_barriers_ret, &[])
			.end()
		));
		setup_commands.execute()
	}));

	// Debug Information //
	let frame_time_ms = RefCell::new(0.0f64);
	let enemy_count = RefCell::new(0u32);
	let debug_info = try!(prelude::DebugInfo::new(&engine, &[
		prelude::DebugLine::Float("Frame Time".to_string(), &frame_time_ms, Some("ms".to_string())),
		prelude::DebugLine::UnsignedInt("Enemy Count".to_string(), &enemy_count, None)
	], &rp_framebuffer_form, 0, swapchain_viewport));

	// Rendering Commands //
	let framebuffer_commands = try!(engine.allocate_graphics_command_buffers(main_frame.get_back_images().len() as u32));
	try!(framebuffer_commands.begin_all().and_then(|iter| iter.map(|(i, recorder)|
	{
		let color_output_barrier = prelude::ImageMemoryBarrier::hold_ownership(
			main_frame.get_back_images()[i], prelude::ImageSubresourceRange::base_color(),
			VK_ACCESS_MEMORY_READ_BIT, VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
			VkImageLayout::PresentSrcKHR, VkImageLayout::ColorAttachmentOptimal);

		recorder
			.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, false, &[], &[],
				&[color_output_barrier])
			.begin_render_pass(&framebuffers[i], &[prelude::AttachmentClearValue::Color(0.0f32, 0.0f32, 0.015625f32, 1.0f32)], false)
			.bind_descriptor_sets(&wire_render_layout, &all_descriptor_sets[0..1])
			.bind_vertex_buffers(&[(&application_data, application_buffer_prealloc.offset(0))])
			.bind_pipeline(background_render)
			.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(2) + structures::background_instance_offs())])
			.push_constants(&wire_render_layout, &[prelude::ShaderStage::Vertex],
				0 .. std::mem::size_of::<f32>() as u32 * 4, &[0.125f32, 0.5f32, 0.1875f32, 0.625f32])
			.draw(4, MAX_BK_COUNT as u32)
			.bind_pipeline(enemy_render)
			.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(2))])
			.push_constants(&wire_render_layout, &[prelude::ShaderStage::Vertex],
				0 .. std::mem::size_of::<f32>() as u32 * 4, &[0.25f32, 0.9875f32, 1.5f32, 1.0f32])
			.draw(4, MAX_ENEMY_COUNT as u32)
			.bind_pipeline(player_render)
			.bind_vertex_buffers_partial(1, &[(&application_data, application_buffer_prealloc.offset(2) + structures::player_instance_offs())])
			.bind_index_buffer(&application_data, application_buffer_prealloc.offset(1))
			.push_constants(&wire_render_layout, &[prelude::ShaderStage::Vertex],
				0 .. std::mem::size_of::<f32>() as u32 * 4, &[1.5f32, 1.25f32, 0.375f32, 1.0f32])
			.draw_indexed(24, 2, 4)
			.inject_commands(|r| debug_info.inject_render_commands(r))
			.end_render_pass()
		.end()
	}).collect::<Result<Vec<_>, _>>()));
	// Transfer Commands //
	let update_commands = try!(engine.allocate_transfer_command_buffers(1));
	try!(update_commands.begin(0).and_then(|recorder|
	{
		let uoffs = application_buffer_prealloc.offset(2);
		let buffer_barriers = [
			prelude::BufferMemoryBarrier::hold_ownership(&application_data, uoffs .. application_buffer_prealloc.total_size(),
				VK_ACCESS_INDEX_READ_BIT | VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT | VK_ACCESS_UNIFORM_READ_BIT, VK_ACCESS_TRANSFER_WRITE_BIT),
			prelude::BufferMemoryBarrier::hold_ownership(&appdata_stage, uoffs .. application_buffer_prealloc.total_size(),
				VK_ACCESS_MEMORY_READ_BIT, VK_ACCESS_TRANSFER_READ_BIT)
		];
		let buffer_barriers_ret = [
			prelude::BufferMemoryBarrier::hold_ownership(&application_data, uoffs .. application_buffer_prealloc.total_size(),
				VK_ACCESS_TRANSFER_WRITE_BIT, VK_ACCESS_INDEX_READ_BIT | VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT | VK_ACCESS_UNIFORM_READ_BIT),
			prelude::BufferMemoryBarrier::hold_ownership(&appdata_stage, uoffs .. application_buffer_prealloc.total_size(),
				VK_ACCESS_TRANSFER_READ_BIT, VK_ACCESS_MEMORY_READ_BIT)
		];

		recorder
			.pipeline_barrier(VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, false, &[], &buffer_barriers, &[])
			.copy_buffer(&appdata_stage, &application_data, &[prelude::BufferCopyRegion(uoffs, uoffs, application_buffer_prealloc.total_size() as usize - uoffs)])
			.pipeline_barrier(VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, false, &[], &buffer_barriers_ret, &[])
		.end()
	}));

	let mut frame_index = try!(main_frame.execute_rendering(&engine, &framebuffer_commands, None, Some(&debug_info), &execute_next_signal));

	let mapped_range = try!(appdata_stage.map());
	let mapped_uniform_data = mapped_range.map_mut::<structures::UniformMemory>(application_buffer_prealloc.offset(3));
	let mapped_instance_data = mapped_range.map_mut::<structures::InstanceMemory>(application_buffer_prealloc.offset(2));
	let (_, uref_enemy, uref_bk, uref_player_center) = mapped_uniform_data.partial_borrow();
	let (iref_enemy, iref_bk, iref_player) = mapped_instance_data.partial_borrow();
	let mut background_datastore = logical_resources::BackgroundDatastore::new(uref_bk, iref_bk);
	let enemy_datastore = RefCell::new(logical_resources::EnemyDatastore::new(uref_enemy, iref_enemy));

	// double-buffered enemy entity list //
	let mut enemy_entities: LinkedList<Enemy> = LinkedList::new();
	let mut player = Player::new(uref_player_center, iref_player);

	let mut secs_from_last_fixed = 0.0f32;
	let mut input = try!(InputSystem::new())
		.add_input(LogicalInputTypes::Horizontal, InputType::Axis(AbsoluteAxisEvents::X))
		.add_input(LogicalInputTypes::Horizontal, InputType::KeyAsAxis(KeyEvents::Left, KeyEvents::Right))
		.add_input(LogicalInputTypes::Vertical, InputType::Axis(AbsoluteAxisEvents::Y))
		.add_input(LogicalInputTypes::Vertical, InputType::KeyAsAxis(KeyEvents::Up, KeyEvents::Down))
		.add_input(LogicalInputTypes::Shoot, InputType::Key(KeyEvents::ButtonA))
		.add_input(LogicalInputTypes::Shoot, InputType::Key(KeyEvents::Z))
		.add_input(LogicalInputTypes::Slowdown, InputType::Axis(AbsoluteAxisEvents::RZ))
		.add_input(LogicalInputTypes::Slowdown, InputType::Key(KeyEvents::ButtonX))
		.add_input(LogicalInputTypes::Slowdown, InputType::Key(KeyEvents::X))
		.add_input(LogicalInputTypes::Overdrive, InputType::Axis(AbsoluteAxisEvents::Z));
	let mut randomizer = rand::thread_rng();
	let background_appear_rate = rand::distributions::Range::new(0, 6);
	let enemy_appear_rate = rand::distributions::Range::new(0, 40);
	let enemy_left_range = rand::distributions::Range::new(-25.0f32, 25.0f32);
	let mut background_next_appear = false;
	let mut enemy_next_appear = false;
	let mut prev_time = time::PreciseTime::now();
	while engine.process_messages()
	{
		// Render code...
		if execute_next_signal.get_status().is_ok()
		{
			let delta_time = prev_time.to(time::PreciseTime::now());
			*frame_time_ms.borrow_mut() = delta_time.num_microseconds().unwrap_or(-1) as f64 / 1000.0f64;
			frame_index = try!
			{
				execute_next_signal.clear().and_then(|()|
				main_frame.present(&engine, frame_index).and_then(|()|
				main_frame.execute_rendering(&engine, &framebuffer_commands, Some(&update_commands), Some(&debug_info), &execute_next_signal)))
			};

			// normal update
			input.update();
			let timescale = (1.0f32 + input[LogicalInputTypes::Slowdown] * 2.0f32) / (1.0f32 + input[LogicalInputTypes::Overdrive]);
			let delta_time_sec = (delta_time.num_milliseconds() as f32 / 1000.0f32) / timescale;
			secs_from_last_fixed += delta_time_sec;
			background_datastore.update(&mut randomizer, delta_time_sec, background_next_appear);

			if enemy_next_appear
			{
				if Enemy::new(&enemy_datastore, enemy_left_range.ind_sample(&mut randomizer)).map(|e| enemy_entities.push_back(e)) == None
				{
					warn!("Enemy Datastore is full!!");
				}
				else { *enemy_count.borrow_mut() += 1; }
				enemy_next_appear = false;
			}
			fn process_2<'a, F>(mut livings: LinkedList<Enemy<'a>>, mut purged_after: LinkedList<Enemy<'a>>,
				enemy_decrease_cb: F, delta_time_sec: f32) -> LinkedList<Enemy<'a>> where F: Fn()
			{
				if let Some(died_e) = purged_after.pop_front() { died_e.die(); }
				let mut purge_index: Option<usize> = None;
				for (idx, e) in purged_after.iter_mut().enumerate()
				{
					if e.update(delta_time_sec)
					{
						enemy_decrease_cb();
						purge_index = Some(idx);
						break;
					}
				}
				if let Some(purge_index) = purge_index
				{
					let mut purged_before = purged_after;
					let purged_after = purged_before.split_off(purge_index);
					livings.append(&mut purged_before);
					process_2(livings, purged_after, enemy_decrease_cb, delta_time_sec)
				}
				else
				{
					livings.append(&mut purged_after);
					livings
				}
			}
			let mut purge_index: Option<usize> = None;
			for (idx, e) in enemy_entities.iter_mut().enumerate()
			{
				if e.update(delta_time_sec)
				{
					*enemy_count.borrow_mut() -= 1;
					purge_index = Some(idx);
					break;
				}
			}
			if let Some(purge_index) = purge_index
			{
				let purged_after = enemy_entities.split_off(purge_index);
				enemy_entities = process_2(enemy_entities, purged_after, || { *enemy_count.borrow_mut() -= 1; }, delta_time_sec);
			}
			player.update(delta_time_sec, &input);

			background_next_appear = false;
			prev_time = time::PreciseTime::now();
		}

		if secs_from_last_fixed >= 1.0f32 / 60.0f32
		{
			// fixed update
			background_next_appear = background_appear_rate.ind_sample(&mut randomizer) == 0;
			enemy_next_appear = enemy_appear_rate.ind_sample(&mut randomizer) == 0;
			secs_from_last_fixed = 0.0f32;
		}
	}
	try!(engine.wait_device());

	Ok(())
}
