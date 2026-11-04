//! Optional transcripts at complete-output boundaries.
use libtest_mimic::{Failed, Trial};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf, sync::Arc};

#[expect(
    clippy::disallowed_types,
    reason = "The thread-local trial context owns one optional I/O writer"
)]
use std::cell::RefCell;
#[expect(
    clippy::disallowed_types,
    reason = "The transcript writer serializes shared record and key I/O"
)]
use std::sync::Mutex;

thread_local! {
    #[expect(clippy::disallowed_types, reason = "Each runner thread selects its own trial's I/O writer")]
    static CURRENT: RefCell<Option<Arc<Context>>> = const { RefCell::new(None) };
}
#[derive(Debug, Default)]
struct State {
    counts: BTreeMap<String, usize>,
    case_step: Option<usize>,
    records: BTreeMap<String, Vec<Value>>,
    uncaptured: bool,
    sequence: usize,
}
#[derive(Debug)]
pub(super) struct Context {
    directory: PathBuf,
    name: String,
    #[expect(
        clippy::disallowed_types,
        reason = "Serialize record writes and key allocation across trial workers"
    )]
    state: Mutex<State>,
}
#[expect(
    clippy::disallowed_methods,
    reason = "Optional test transcript boundary reads its destination"
)]
#[expect(
    clippy::disallowed_types,
    reason = "Construct the one shared writer state for a selected trial"
)]
pub(super) fn trial<R>(name: impl Into<String>, run: R) -> Trial
where
    R: FnOnce() -> Result<(), Failed> + Send + 'static,
{
    let name = name.into();
    Trial::test(name.clone(), move || {
        let context = std::env::var_os("BIOMCP_TRANSCRIPT_DIR").map(|directory| {
            Arc::new(Context {
                directory: directory.into(),
                name,
                state: Mutex::new(State::default()),
            })
        });
        CURRENT.with(|current| *current.borrow_mut() = context.clone());
        let result = run();
        CURRENT.with(|current| *current.borrow_mut() = None);
        if let Some(context) = context {
            let state = context
                .state
                .lock()
                .map_err(|e| Failed::from(e.to_string()))?;
            context
                .write(
                    &format!("trial/{}", context.name),
                    &json!({
                        "kind":"trial", "name":context.name,
                        "uncaptured":state.uncaptured || state.records.is_empty(),
                    }),
                )
                .map_err(Failed::from)?;
        }
        result
    })
}
pub(super) fn current() -> Option<Arc<Context>> {
    CURRENT.with(|current| current.borrow().clone())
}
pub(super) fn case_step(index: usize) {
    if let Some(context) = current() {
        context
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .case_step = Some(index);
    }
}
pub(super) use super::stable_hash::hash;
impl Context {
    fn write(&self, key: &str, value: &Value) -> Result<(), String> {
        fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
        fs::write(
            self.directory
                .join(format!("{}.json", hash(key.as_bytes()))),
            bytes,
        )
        .map_err(|e| e.to_string())
    }
    pub(super) fn sequence(&self) -> usize {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let sequence = state.sequence;
        state.sequence += 1;
        sequence
    }
    pub(super) fn allocate(&self, args: &[String]) -> String {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(index) = state.case_step {
            return format!("case/{}/step/{index}", self.name);
        }
        let digest = hash(&serde_json::to_vec(args).unwrap_or_default());
        let count = state.counts.entry(digest.clone()).or_default();
        let key = format!("trial/{}/{digest}/{}", self.name, *count);
        *count += 1;
        key
    }
    pub(super) fn uncaptured(&self) {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .uncaptured = true;
    }
    pub(super) fn record(&self, key: &str, mut execution: Value) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        if let Some(object) = execution.as_object_mut()
            && object.contains_key("sequence_start")
        {
            object.insert("sequence_end".into(), json!(state.sequence));
            state.sequence += 1;
        }
        let executions = state.records.entry(key.into()).or_default();
        executions.push(execution);
        self.write(key, &json!({"key":key,"executions":executions}))
    }
}
