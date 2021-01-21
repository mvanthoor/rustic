/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

use super::Engine;
use crate::{comm::CommControl, search::defs::SearchReport};

impl Engine {
    pub fn search_reports(&mut self, search_report: &SearchReport) {
        match search_report {
            SearchReport::Finished(m) => {
                self.comm.send(CommControl::BestMove(*m));
                self.comm.send(CommControl::Update);
            }

            SearchReport::SearchCurrentMove(curr_move) => {
                self.comm.send(CommControl::SearchCurrMove(*curr_move));
            }

            SearchReport::SearchSummary(summary) => {
                self.comm.send(CommControl::SearchSummary(summary.clone()));
            }

            SearchReport::SearchStats(stats) => {
                self.comm.send(CommControl::SearchStats(*stats));
            }
        }
    }
}
