use super::SubworkflowRunnerShell;
use crate::store::WorkflowRepository;

type OrderedMap<K, V> = std::collections::BTreeMap<K, V>;
type HashMap<K, V> = std::collections::HashMap<K, V>;
type Arc<T> = std::sync::Arc<T>;
type Workflow = crate::engine::Workflow;
type Executor = crate::engine::executor::Executor;
type NodeStatus = crate::engine::executor::NodeStatus;
type StoreError = crate::store::StoreError;

pub struct WorkflowCatalogRepository {
    workflows: std::sync::RwLock<OrderedMap<String, Workflow>>,
}

impl WorkflowCatalogRepository {
    pub fn from_repository(repository: &impl WorkflowRepository) -> Result<Self, StoreError> {
        let workflow_names = repository.list_workflows()?;
        let workflows = workflow_names
            .into_iter()
            .map(|name| {
                repository
                    .load_workflow(&name)?
                    .ok_or_else(|| {
                        StoreError::NotFound("listed workflow disappeared while building execution catalog".to_string())
                    })
                    .map(|workflow| (name, workflow))
            })
            .collect::<Result<OrderedMap<_, _>, StoreError>>()?;
        Ok(Self {
            workflows: std::sync::RwLock::new(workflows),
        })
    }
}

impl WorkflowRepository for WorkflowCatalogRepository {
    fn save_workflow(&self, workflow: &Workflow) -> Result<(), StoreError> {
        self.workflows
            .write()
            .map_err(|_| StoreError::Db("workflow catalog write lock poisoned".to_string()))?
            .insert(workflow.meta.name.clone(), workflow.clone());
        Ok(())
    }

    fn load_workflow(&self, name: &str) -> Result<Option<Workflow>, StoreError> {
        Ok(self
            .workflows
            .read()
            .map_err(|_| StoreError::Db("workflow catalog read lock poisoned".to_string()))?
            .get(name)
            .cloned())
    }

    fn list_workflows(&self) -> Result<Vec<String>, StoreError> {
        Ok(self
            .workflows
            .read()
            .map_err(|_| StoreError::Db("workflow catalog read lock poisoned".to_string()))?
            .keys()
            .cloned()
            .collect())
    }

    fn delete_workflow(&self, name: &str) -> Result<(), StoreError> {
        self.workflows
            .write()
            .map_err(|_| StoreError::Db("workflow catalog write lock poisoned".to_string()))?
            .remove(name);
        Ok(())
    }
}

pub struct RepositorySubworkflowRunnerShell<R> {
    repository: Arc<R>,
}

impl<R> RepositorySubworkflowRunnerShell<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait::async_trait]
impl<R> SubworkflowRunnerShell for RepositorySubworkflowRunnerShell<R>
where R: WorkflowRepository + Send + Sync
{
    // r[impl n8nclone.workflow_execution.subworkflows.events]
    async fn execute(
        &self,
        invocation: &super::SubworkflowInvocation,
    ) -> Result<super::SubworkflowExecution, super::SubworkflowExecutionError> {
        let workflow = self
            .repository
            .load_workflow(&invocation.workflow_name)
            .map_err(|_| super::SubworkflowExecutionError::ChildWorkflowLoadFailed)?
            .ok_or_else(|| super::SubworkflowExecutionError::ChildWorkflowMissing(invocation.workflow_name.clone()))?;
        crate::control_flow::validate_subworkflow_admission(
            &crate::control_flow::SubworkflowNodeConfig {
                workflow_name: workflow.meta.name.clone(),
                input_mappings: Vec::new(),
                output_mappings: Vec::new(),
            },
            &invocation.ancestry,
        )
        .map_err(|_| super::SubworkflowExecutionError::WorkflowIdentityMismatch)?;

        let mut executor = Executor::new();
        let (child_run_id, steps) =
            executor.run_with_named_inputs(&workflow, &HashMap::new(), &invocation.inputs).await?;
        if steps.iter().any(|step| matches!(step.status, NodeStatus::Failed { .. })) {
            return Err(super::SubworkflowExecutionError::ChildWorkflowExecutionFailed);
        }
        let outputs = super::collect_requested_outputs(&workflow, &steps, &invocation.requested_outputs)?;
        let metadata = super::SubworkflowEventMetadata {
            parent_run_id: invocation.correlation.parent_run_id,
            subworkflow_node_id: invocation.correlation.subworkflow_node_id,
            child_run_id,
            workflow_name: invocation.workflow_name.clone(),
        };
        Ok(super::SubworkflowExecution {
            workflow_name: invocation.workflow_name.clone(),
            child_run_id,
            outputs,
            events: vec![
                super::SubworkflowEvent {
                    kind: super::SubworkflowEventKind::Started,
                    metadata: metadata.clone(),
                },
                super::SubworkflowEvent {
                    kind: super::SubworkflowEventKind::Completed,
                    metadata,
                },
            ],
        })
    }
}
