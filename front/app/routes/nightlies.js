import Ember from 'ember';
import fetch from 'fetch';
import ENV from 'rust-dashboard/config/environment';

function fixTimestamps(data) {
  return data.map(elt => {
    return [elt[0] * 1000, elt[1]];
  });
}

export default Ember.Route.extend({
  model() {
    const summary_url = `${ENV.apiBaseURL}releases`;
    return fetch(summary_url)
      .then(response => response.json())
      .then(({ streak_summary, nightlies, builder_times_mins }) => ({
        streak: streak_summary,
        nightlies: nightlies.map(elt => {
          return {
            nightly: elt[0],
            builds: elt[1],
          };
        }),
        build_times: builder_times_mins.map(series => ({ 
          name: series[0],
          data: fixTimestamps(series[1])
        }))
      }));
  }
});
