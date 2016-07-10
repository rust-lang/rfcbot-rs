import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

function fixTimestamps(data) {
  return data.map(elt => {
    return [elt[0] * 1000, elt[1]];
  });
}

export default Ember.Route.extend({
  model: function() {
    const summary_url = `${ENV.apiBaseURL}releases`;
    return Ember.$.getJSON(summary_url)
      .then(metrics => {
        return {
          streak: metrics.streak_summary,
          nightlies: metrics.nightlies.map(elt => {
            return {
              nightly: elt[0],
              builds: elt[1],
            };
          }),
          build_times: metrics.builder_times_mins.map(series => {
            return { name: series[0], data: fixTimestamps(series[1]) };
          })
        };
      });
  }
});
