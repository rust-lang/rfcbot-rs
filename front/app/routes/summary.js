import Ember from 'ember';

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
    return { stable: stableReleases() }; }
});
