use std::{collections::HashMap, hash::Hash};

pub fn combine_errors<T, U, E>(
    result1: Result<T, Vec<E>>,
    result2: Result<U, Vec<E>>,
) -> Result<(T, U), Vec<E>> {
    match (result1, result2) {
        (Ok(result1), Ok(result2)) => Ok((result1, result2)),
        (Err(errors), Ok(_)) => Err(errors),
        (Ok(_), Err(errors)) => Err(errors),
        (Err(errors1), Err(errors2)) => Err(errors1.into_iter().chain(errors2).collect()),
    }
}

pub fn combine_error_list<T, E>(
    results: impl IntoIterator<Item = Result<T, Vec<E>>>,
) -> Result<Vec<T>, Vec<E>> {
    results
        .into_iter()
        .fold(Ok(Vec::new()), |acc, result| match acc {
            Ok(mut acc) => match result {
                Ok(result) => {
                    acc.push(result);
                    Ok(acc)
                }
                Err(errors) => Err(errors),
            },
            Err(mut acc_errors) => match result {
                Ok(_result) => Err(acc_errors),
                Err(errors) => {
                    acc_errors.extend(errors);
                    Err(acc_errors)
                }
            },
        })
}

pub fn split_ok_and_errors<T, I, E, O>(
    results: impl IntoIterator<Item = Result<T, (I, Vec<E>)>>,
) -> (O, HashMap<I, Vec<E>>)
where
    I: Eq + Hash,
    O: FromIterator<T>,
{
    let (ok, errors) = results.into_iter().fold(
        (Vec::new(), HashMap::new()),
        |(mut ok, mut acc_errors), result| match result {
            Ok(result) => {
                ok.push(result);
                (ok, acc_errors)
            }
            Err((key, errors)) => {
                assert!(!acc_errors.contains_key(&key), "duplicate error");
                acc_errors.insert(key, errors);
                (ok, acc_errors)
            }
        },
    );

    let ok = ok.into_iter().collect();
    let errors = errors.into_iter().collect();

    (ok, errors)
}

pub fn convert_errors<E1, E2>(errors: Vec<E1>) -> Vec<E2>
where
    E1: Into<E2>,
{
    errors.into_iter().map(|error| error.into()).collect()
}
