import Ember from 'ember';

function stableReleases() {
  var previousDate = new Date('2015-12-11');
  var nextDate = new Date('2016-01-22');
  var nextNextDate = new Date('2016-02-04');

  var prevRelease = 5;
  var nextRelease = 6;
  var nextNextRelease = 7;

  while (Date.now() > nextDate) {
    previousDate = new Date(nextDate);
    nextDate.setDate(nextDate.getDate() + (7 * 6));
    nextNextDate.setDate(nextDate.getDate() + (7 * 6)); // yay mutable state

    prevRelease += 1;
    nextRelease += 1;
    nextNextRelease += 1;
  }

  return {
    previous_date: previousDate.toDateString(),
    next_date: nextDate.toDateString(),
    next_next_date: nextNextDate.toDateString(),
    previous_version: prevRelease,
    next_version: nextRelease,
    next_next_version: nextNextRelease
  };
}

export default Ember.Route.extend({
  model: function() {
    return { stable: stableReleases() };
  }
});
