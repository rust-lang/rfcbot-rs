import Ember from 'ember';
import ENV from 'rust-dashboard/config/environment';

function stableReleases() {
  var previousDate = new Date('2015-12-11');
  var nextDate = new Date('2016-01-22');

  var prevRelease = 5;
  var nextRelease = 6;

  while (Date.now() > nextDate) {
    previousDate = new Date(nextDate);
    nextDate.setDate(nextDate.getDate() + (7 * 6));

    prevRelease += 1;
    nextRelease += 1;
  }

  return {
    previous_date: previousDate.toDateString(),
    next_date: nextDate.toDateString(),
    previous_version: prevRelease,
    next_version: nextRelease
  };
}

export default Ember.Route.extend({
  model: function() {
    const summary_url = `${ENV.apiBaseURL}releases`;
    return Ember.$.getJSON(summary_url)
      .then(metrics => {
        return {
          stable: stableReleases(),
          nightlies: metrics.nightlies.map(elt => {
            return {
              nightly: elt[0],
              builds: elt[1],
            };
          })
        };
      });
  }
});
