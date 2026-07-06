use std::collections::HashSet;

use heck::ToSnakeCase;
use proc_macro2::Span;
use syn::Ident;
use syn::spanned::Spanned;

use crate::App;
use crate::parser::SubApp;
use crate::parser::ast::{HardwareTask, SharedResources};

pub struct Analysis {
    pub sub_analysis: Vec<SubAnalysis>,
    pub task_traits: HashSet<syn::Ident>,
}

impl Analysis {
    /// - updates resource ceilings
    /// - collects and structure key information about the user application to be used during code generation
    /// - collect the task traits
    pub fn run(parsed_app: &mut App) -> syn::Result<Self> {
        // update resource ceilings
        for app in parsed_app.sub_apps.iter_mut() {
            update_resource_priorities(app.shared.as_mut(), &app.tasks)?;

            if let Some(sr) = app.shared.as_ref() {
                print_shared_ress(sr);
            }
        }

        // collect and structure key information about the user application to be used during code generation
        let sub_analysis = parsed_app
            .sub_apps
            .iter()
            .map(SubAnalysis::run)
            .collect::<syn::Result<_>>()?;

        let mut task_traits = HashSet::new();
        for subapp in parsed_app.sub_apps.iter() {
            for task in subapp.tasks.iter() {
                task_traits.insert(task.args.task_trait.clone());
            }
            if let Some(idle) = &subapp.idle {
                task_traits.insert(idle.args.task_trait.clone());
            }
        }

        Ok(Self {
            sub_analysis,
            task_traits,
        })
    }
}

/// Print shared resources
fn print_shared_ress(sr: &SharedResources) {
    let ress = &sr.resources;
    let res_names = ress.iter().map(|res| res.ident.to_string());
    let longest = unsafe {
        res_names
            .clone()
            .map(|s| s.chars().count())
            .max()
            .unwrap_unchecked()
    };
    println!("[RTIC] Shared resources");
    for res in ress {
        let pi = res.priority;
        println!("[RTIC] * {:<longest$} @π={pi}", res.ident);
    }
}

#[derive(Debug)]
pub struct SubAnalysis {
    // used interrupts and their priorities
    // HACK: u32 instead of u16 to support EDFIC deadlines
    pub used_irqs: Vec<(syn::Ident, u32)>,
    // tasks requiring some late local resource initialization.
    pub late_resource_tasks: Vec<LateResourceTask>,
}

impl SubAnalysis {
    pub fn run(app: &SubApp) -> syn::Result<Self> {
        // hw interrupts bound to hardware tasks
        let used_interrupts: Vec<(Ident, u32)> = app
            .tasks
            .iter()
            .filter_map(|t| Some((t.args.binds.clone()?, t.args.priority.into())))
            .collect();
        print_irqs(&used_interrupts);

        let user_initializable_tasks = app
            .tasks
            .iter()
            .chain(app.idle.iter()) // idle is also a task and we shouldn't forget it
            .filter_map(|t| {
                if t.user_initializable {
                    Some(LateResourceTask {
                        task_name: t.task_struct.ident.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(Self {
            used_irqs: used_interrupts,
            late_resource_tasks: user_initializable_tasks,
        })
    }
}

fn print_irqs(irqs: &[(Ident, u32)]) {
    if irqs.len() != 0 {
        println!("[RTIC] Interrupts:");
        let irq_names = irqs.iter().map(|(irq, _)| irq.to_string());
        let longest = unsafe {
            irq_names
                .clone()
                .map(|s| s.chars().count())
                .max()
                .unwrap_unchecked()
        };
        let mut it: Vec<_> = irqs
            .iter()
            .zip(irq_names)
            .map(|((_, prio), irq_name)| (irq_name, prio))
            .collect();
        it.sort_by_key(|&(_, prio)| prio);

        // Print IRQs in priority order (highest to lowest)
        for (irq, prio) in it.iter().rev() {
            println!("[RTIC] * {irq:<longest$} @p={prio}");
        }
    }
}

fn update_resource_priorities(
    shared: Option<&mut SharedResources>,
    hw_tasks: &[HardwareTask],
) -> syn::Result<()> {
    let Some(shared) = shared else { return Ok(()) };
    for task in hw_tasks.iter() {
        let task_priority = task.args.priority;
        for resource_ident in task.args.shared.iter() {
            // Go trough all shared resources of the tasks
            if let Some(shared_element) = shared.get_field_mut(resource_ident) {
                // increase the shared_element's priority to the highest priority task that has it as a shared resource
                if shared_element.priority < task_priority {
                    shared_element.priority = task_priority
                }

                // increase the shared_element's read_priority to the highest priority task that has it as a shared resource
                if shared_element.read_priority < task_priority {
                    shared_element.read_priority = task_priority
                }
            } else {
                return Err(syn::Error::new(
                    task.task_struct.span(),
                    format!(
                        "The resource `{resource_ident}` was not found in `{}`",
                        shared.strct.ident
                    ),
                ));
            }
        }
        for resource_ident in task.args.read.iter() {
            // Go trough all read resources of the tasks
            if let Some(shared_element) = shared.get_field_mut(resource_ident) {
                // increase the shared_element's priority to the highest priority task that has it as a shared OR read resource
                if shared_element.priority < task_priority {
                    shared_element.priority = task_priority
                }
            } else {
                return Err(syn::Error::new(
                    task.task_struct.span(),
                    format!(
                        "The resource `{resource_ident}` was not found in `{}`",
                        shared.strct.ident
                    ),
                ));
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct LateResourceTask {
    pub task_name: Ident,
}
impl LateResourceTask {
    /// By convention, this method is used to generate the name of the static task instance
    pub fn name_uppercase(&self) -> Ident {
        let name = self.task_name.to_string().to_snake_case().to_uppercase();
        Ident::new(&name, Span::call_site())
    }

    pub fn name_snakecase(&self) -> Ident {
        let name = self.task_name.to_string().to_snake_case();
        Ident::new(&name, Span::call_site())
    }
}
