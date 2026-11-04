//! NCI STRAP glossary, Study Design > Design > Phase, publishes this complete LOV.
// https://clinicaltrialsapi.cancer.gov/glossary
macro_rules! phases {
    ($($values:tt)*) => {
        biodata::native_enum! {
            /// Exact NCI response phases, retaining future publisher values.
            NciTrialPhase { $($values)* }
        }
        biodata::wire_enum! {
            /// Published NCI phase filters; future values require explicit support.
            NciPhaseFilter { $($values)* }
        }
    };
}
phases! {
        /// Publisher NA label.
        NotApplicable => "NA",
        /// Publisher phase 0 label.
        Phase0 => "0",
        /// Publisher phase I label.
        Phase1 => "I",
        /// Publisher phase I_II label.
        Phase1And2 => "I_II",
        /// Publisher phase II label.
        Phase2 => "II",
        /// Publisher phase II_III label.
        Phase2And3 => "II_III",
        /// Publisher phase III label.
        Phase3 => "III",
        /// Publisher phase IV label.
        Phase4 => "IV",
}
