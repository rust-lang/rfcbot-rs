import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

export default Ember.Route.extend({
  model() {
    const url = `${ENV.apiBaseURL}fcp/all`;
    return fetch(url)
      .then(response => response.json())
      .then((fcps) => {
        var teams = new Map();

        // go over each FCP proposal
        for (let fcp of fcps) {

            var missingReviews = [];

            for (let review of fcp.reviews) {
                if (!review[1]) {
                    missingReviews.push(review[0].login);
                }
            }

            missingReviews.sort();

            var record = {
                disposition: fcp.fcp.disposition,
                issue: fcp.issue,
                statusComment: fcp.status_comment,
                pendingReviewers: missingReviews
            };

            // insert this record for all the teams
            for (let label of fcp.issue.labels) {

                // we only care about team labels
                if (label.startsWith("T-")) {

                    if (!teams.has(label)) {
                        teams.set(label, []);
                    }

                    teams.get(label).push(record);
                }
            }
        }

        var toReturn = [];

        for (let pair of teams) {
            toReturn.push({
                team: pair[0],
                fcps: pair[1]
            });
        }

        return { fcps: toReturn };
      });
  }
});
