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
    const summary_url = ENV.apiBaseURL + 'issues';
    return fetch(summary_url)
      .then(metrics => {

        const issues_open_per_day = fixTimestamps(metrics.opened_per_day);
        const issues_closed_per_day = fixTimestamps(metrics.closed_per_day);
        const issues_days_open_b4_close = fixTimestamps(metrics.days_open_before_close);

        const model = {
          issues: {
            days_open_current_mean: metrics.current_open_age_days_mean.toFixed(2),
            num_high_priority: metrics.num_open_p_high_issues,
            num_nightly_regress: metrics.num_open_regression_nightly_issues,
            num_beta_regress: metrics.num_open_regression_beta_issues,
            num_stable_regress: metrics.num_open_regression_stable_issues,
            open_close_per_day: [{
              name: 'Issues Opened Per Day',
              data: issues_open_per_day
            }, {
              name: 'Issues Closed Per Day',
              data: issues_closed_per_day
            }],
            days_open_before_close: [{
              name: 'Issues Days Open Before Closed (by week)',
              data: issues_days_open_b4_close
            }]
          }
        };

        return model;
      });
  }
});
