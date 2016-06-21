#![allow(non_upper_case_globals)]

use std::collections::BTreeMap;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
struct Member {
    username: &'static str,
    full_name: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Team {
    name: &'static str,
    label: &'static str,
    ping: &'static str,
}

static aatch: Member = Member {
    username: "aatch",
    full_name: "James Miller",
};
static arielb1: Member = Member {
    username: "arielb1",
    full_name: "Ariel Ben-Yehuda",
};
static acrichto: Member = Member {
    username: "alexcrichton",
    full_name: "Alex Crichton",
};
static aturon: Member = Member {
    username: "aturon",
    full_name: "Aaron Turon",
};
static bkoropoff: Member = Member {
    username: "bkoropoff",
    full_name: "Brian Koropoff",
};
static brson: Member = Member {
    username: "brson",
    full_name: "Brian Anderson",
};
// static bstrie: Member = Member { username: "bstrie", full_name: "Ben Striegel" };
static burntsushi: Member = Member {
    username: "BurntSushi",
    full_name: "Andrew Gallant",
};
// static carols10cents: Member = Member { username: "carols10cents", full_name: "Carol Nichols" };
static dotdash: Member = Member {
    username: "dotdash",
    full_name: "Björn Steinbrink",
};
static eddyb: Member = Member {
    username: "eddyb",
    full_name: "Eduard Burtescu",
};
static erickt: Member = Member {
    username: "erickt",
    full_name: "Erick Tryzelaar",
};
static gankro: Member = Member {
    username: "Gankro",
    full_name: "Alexis Beingessner",
};
static huonw: Member = Member {
    username: "huonw",
    full_name: "Huon Wilson",
};
static kimundi: Member = Member {
    username: "Kimundi",
    full_name: "Marvin Löbel",
};
// static manishearth: Member = Member { username: "Manishearth", full_name: "Manish Goregaokar" };
// static mbrubeck: Member = Member { username: "mbrubeck", full_name: "Matt Brubeck" };
static michaelwoerister: Member = Member {
    username: "michaelwoerister",
    full_name: "Michael Woerister",
};
// static niconii: Member = Member { username: "niconii", full_name: "Nicolette Verlinden" };
static nikomatsakis: Member = Member {
    username: "nikomatsakis",
    full_name: "Niko Matsakis",
};
static nrc: Member = Member {
    username: "nrc",
    full_name: "Nick Cameron",
};
static pcwalton: Member = Member {
    username: "pcwalton",
    full_name: "Patrick Walton",
};
static pnkfelix: Member = Member {
    username: "pnkfelix",
    full_name: "Felix Klock",
};
static sfackler: Member = Member {
    username: "sfackler",
    full_name: "Steven Fackler",
};
// static skade: Member = Member { username: "skade", full_name: "Florian Gilcher" };
static steveklabnik: Member = Member {
    username: "steveklabnik",
    full_name: "Steve Klabnik",
};
static vadimcn: Member = Member {
    username: "vadimcn",
    full_name: "Vadim Chugunov",
};
static wycats: Member = Member {
    username: "wycats",
    full_name: "Yehuda Katz",
};

lazy_static! {
// no community or mod team b/c they won't have nags
    pub static ref MEMBERSHIP: BTreeMap<&'static str, (&'static str, Vec<Team>)> = {
        let mut teams = BTreeMap::new();

        teams.insert(Team { name: "Core", label: "T-core", ping: "@rust-lang/core" },
            vec![&brson, &acrichto, &wycats, &steveklabnik, &nikomatsakis,
                 &aturon, &pcwalton, &huonw, &erickt]);

        teams.insert(Team { name: "Language design", label: "T-lang", ping: "@rust-lang/lang"},
            vec![&eddyb, &nrc, &pnkfelix, &nikomatsakis, &aturon, &huonw]);

        teams.insert(Team { name: "Library", label: "T-libs", ping: "@rust-lang/libs"},
            vec![&brson, &gankro, &acrichto, &sfackler, &burntsushi, &kimundi, &aturon, &huonw]);

        teams.insert(Team { name: "Compiler", label: "T-compiler", ping: "@rust-lang/compiler" },
            vec![&arielb1, &eddyb, &nrc, &pnkfelix, &bkoropoff, &nikomatsakis, &aatch,
                 &dotdash, &michaelwoerister]);

        teams.insert(Team { name: "Tooling and infrastructure",
                            label: "T-tools",
                            ping: "@rust-lang/tools" },
            vec![&brson, &nrc, &acrichto, &vadimcn, &wycats, &michaelwoerister]);

        // while the above definition is in terms of teams, we want to look up by member username
        let mut membership = BTreeMap::new();

        for (team, members) in teams {
            for member in members {
                let &mut (_, ref mut member_of) = membership
                    .entry(member.username).or_insert((member.full_name, vec![]));

                member_of.push(team)
            }
        }

        membership
    };
}
