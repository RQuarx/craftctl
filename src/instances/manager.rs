use crate::result::Result;

use super::{CreateInstanceRequest, Instance, InstanceStorage, instance::instance_id_from_name};

#[derive(Debug, Clone)]
pub struct InstanceManager {
    storage: InstanceStorage,
}

impl InstanceManager {
    pub fn create(storage: InstanceStorage) -> Self {
        Self { storage }
    }

    pub fn with_default_storage() -> Result<Self> {
        Ok(Self::create(InstanceStorage::default()?))
    }

    pub fn load_instances(&self) -> Result<Vec<Instance>> {
        self.storage.load_all()
    }

    pub fn create_instance(&self, request: CreateInstanceRequest) -> Result<Instance> {
        let id = self.next_available_id(&request.name);
        let instance = Instance::create(id, request)?;

        self.storage.save(&instance)?;

        Ok(instance)
    }

    pub fn update_instance(
        &self,
        mut instance: Instance,
        request: CreateInstanceRequest,
    ) -> Result<Instance> {
        instance.update(request)?;
        self.storage.save(&instance)?;

        Ok(instance)
    }

    pub fn delete_instance(&self, instance: &Instance) -> Result<()> {
        self.storage.delete(&instance.id)
    }

    pub fn storage(&self) -> &InstanceStorage {
        &self.storage
    }

    fn next_available_id(&self, name: &str) -> String {
        let base_id = instance_id_from_name(name);

        if !self.storage.exists(&base_id) {
            return base_id;
        }

        for suffix in 2.. {
            let candidate = format!("{base_id}-{suffix}");

            if !self.storage.exists(&candidate) {
                return candidate;
            }
        }

        unreachable!("unbounded suffix search should always find an available id")
    }
}
