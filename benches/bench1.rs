//! This benchmark is used to compare the time it takes to create
//! * borrowing MownStr's vs. standard &str references
//! * owning MownStr's vs. Strings
//!
//! The results of `borrowed_mownstr` should therefore be compared to `refs`,
//! and that of `owned_mownstr` should be compared to `strings`.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mownstr::MownStr;

fn refs(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("refs", ""),
        black_box(&STRINGS),
        |b, &i| {
            b.iter(|| {
                let v = i.iter().cloned().collect::<Vec<_>>();
                assert!(v.len() == i.len())
            });
        },
    );
}

fn borrowed_mownstr(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("borrowed_mownstr", ""),
        black_box(&STRINGS),
        |b, &i| {
            b.iter(|| {
                let v = i.iter().map(|r| MownStr::from(*r)).collect::<Vec<_>>();
                assert!(v.len() == i.len())
            });
        },
    );
}

fn strings(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("strings", ""),
        black_box(&STRINGS),
        |b, &i| {
            b.iter(|| {
                let v = i
                    .iter()
                    .map(|r| r.to_string())
                    .map(String::into_boxed_str)
                    .collect::<Vec<_>>();
                assert!(v.len() == i.len())
            });
        },
    );
}

fn owned_mownstr(c: &mut Criterion) {
    c.bench_with_input(
        BenchmarkId::new("owned_mownstr", ""),
        black_box(&STRINGS),
        |b, &i| {
            b.iter(|| {
                let v = i
                    .iter()
                    .map(|r| r.to_string())
                    .map(MownStr::from)
                    .collect::<Vec<_>>();
                assert!(v.len() == i.len())
            });
        },
    );
}

criterion_group!(benches, refs, borrowed_mownstr, strings, owned_mownstr,);
criterion_main!(benches);

const STRINGS: [&'static str; 30] = [
    "Jeunes gens, prenez garde aux choses que vous dites.",
    "Tout peut sortir d'un mot qu'en passant vous perdîtes.",
    "Tout, la haine et le deuil ! - Et ne m'objectez pas",
    "Que vos amis sont sûrs et que vous parlez bas... -",
    "Ecoutez bien ceci :",
    "Tête-à-tête, en pantoufle,",
    "Portes closes, chez vous, sans un témoin qui souffle,",
    "Vous dites à l'oreille au plus mystérieux",
    "De vos amis de coeur, ou, si vous l'aimez mieux,",
    "Vous murmurez tout seul, croyant presque vous taire,",
    "Dans le fond d'une cave à trente pieds sous terre,",
    "Un mot désagréable à quelque individu ;",
    "Ce mot que vous croyez qu'on n'a pas entendu,",
    "Que vous disiez si bas dans un lieu sourd et sombre,",
    "Court à peine lâché, part, bondit, sort de l'ombre !",
    "Tenez, il est dehors ! Il connaît son chemin.",
    "Il marche, il a deux pieds, un bâton à la main,",
    "De bons souliers ferrés, un passeport en règle ;",
    "- Au besoin, il prendrait des ailes, comme l'aigle ! -",
    "Il vous échappe, il fuit, rien ne l'arrêtera.",
    "Il suit le quai, franchit la place, et caetera,",
    "Passe l'eau sans bateau dans la saison des crues,",
    "Et va, tout à travers un dédale de rues,",
    "Droit chez l'individu dont vous avez parlé.",
    "Il sait le numéro, l'étage ; il a la clé,",
    "Il monte l'escalier, ouvre la porte, passe,",
    "Entre, arrive, et, railleur, regardant l'homme en face,",
    "Dit : - Me voilà ! je sors de la bouche d'un tel. -",
    "Et c'est fait. Vous avez un ennemi mortel.",
    "-- poem by Victor Hugo",
];
