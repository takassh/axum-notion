use entity::top::Top;
use shuttle_persist::PersistInstance;

#[derive(Clone, Debug)]
pub struct TopRepository {
    pub instance: PersistInstance,
    key: String,
}

impl TopRepository {
    pub fn new(instance: PersistInstance) -> Self {
        Self {
            instance,
            key: "top".to_string(),
        }
    }
}

impl TopRepository {
    pub fn get(&self) -> anyhow::Result<Top> {
        let value = self.instance.load::<Top>(&self.key)?;
        Ok(value)
    }

    pub fn set(&self, value: &str) -> anyhow::Result<Top> {
        let top = Top::new_from_str(value)?;
        self.instance.save::<Top>(&self.key, top.clone())?;
        Ok(top)
    }
}
