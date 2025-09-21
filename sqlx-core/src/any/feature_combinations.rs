// Shared recursive macro to generate all non-empty combinations of feature flags.
// Pass a list of entries with a feature name and an arbitrary payload which is
// forwarded to the callback when that feature is selected.
//
// Usage:
// for_all_feature_combinations!{
//     entries: [("postgres", Postgres), ("mysql", MySql)],
//     callback: my_callback
// }
// will expand to (for the active feature configuration):
// #[cfg(all(feature="postgres"), not(feature="mysql"))] my_callback!(Postgres);
// #[cfg(all(feature="mysql"), not(feature="postgres"))] my_callback!(MySql);
// #[cfg(all(feature="postgres", feature="mysql"))] my_callback!(Postgres, MySql);
// and so on for all non-empty subsets.
#[macro_export]
macro_rules! for_all_feature_combinations {
    ( entries: [ $( ( $feat:literal, $payload:tt ) ),* $(,)? ], callback: $callback:ident ) => {
        $crate::for_all_feature_combinations!(@recurse [] [] [ $( ( $feat, $payload ) )* ] $callback);
    };

    (@recurse [$($yes:tt)*] [$($no:tt)*] [ ( $feat:literal, $payload:tt ) $($rest:tt)* ] $callback:ident ) => {
        $crate::for_all_feature_combinations!(@recurse [ $($yes)* ( $feat, $payload ) ] [ $($no)* ] [ $($rest)* ] $callback);
        $crate::for_all_feature_combinations!(@recurse [ $($yes)* ] [ $($no)* $feat ] [ $($rest)* ] $callback);
    };

    // Base case: at least one selected
    (@recurse [ $( ( $yfeat:literal, $ypayload:tt ) )+ ] [ $( $nfeat:literal )* ] [] $callback:ident ) => {
        #[cfg(all( $( feature = $yfeat ),+ $(, not(feature = $nfeat ))* ))]
        $callback!( $( $ypayload ),+ );
    };

    // Base case: none selected (skip)
    (@recurse [] [ $( $nfeat:literal )* ] [] $callback:ident ) => {};
}
