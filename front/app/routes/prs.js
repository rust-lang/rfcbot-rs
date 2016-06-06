import Ember from 'ember';
import fetch from 'fetch';
import ENV from 'rust-dashboard/config/environment';

function fixTimestamps(data) {
  return data.map(elt => {
    return [elt[0] * 1000, elt[1]];
  });
}

export default Ember.Route.extend({
  model: function() {
    const summary_url = `${ENV.apiBaseURL}pullrequests`;
    return fetch(summary_url)
      .then(metrics => {
        const prs_open_per_day = fixTimestamps(metrics.opened_per_day);
        const prs_closed_per_day = fixTimestamps(metrics.closed_per_day);
        const prs_merged_per_day = fixTimestamps(metrics.merged_per_day);
        const prs_days_open_b4_close = fixTimestamps(metrics.days_open_before_close);

        const model = {
          days_open_current_mean: metrics.current_open_age_days_mean.toFixed(2),
          open_close_per_day: [{
            name: 'PRs Opened Per Day',
            data: prs_open_per_day
          }, {
            name: 'PRs Closed Per Day',
            data: prs_closed_per_day
          }, {
            name: 'PRs Merged Per Day',
            data: prs_merged_per_day
          }],
          days_open_before_close: [{
            name: 'PR Days Open Before Closed (by week)',
            data: prs_days_open_b4_close
          }]
        };

        return model;
      });
  }
});
