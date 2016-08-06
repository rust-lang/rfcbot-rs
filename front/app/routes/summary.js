import Ember from 'ember';

function stableReleases() {
  let previousDate = new Date('2015-12-11');
  const nextDate = new Date('2016-01-22');
  const nextNextDate = new Date('2016-03-04');

  let prevRelease = 5;
  let nextRelease = 6;
  let nextNextRelease = 7;

  while (Date.now() > nextDate) {
    previousDate = new Date(nextDate);
    nextDate.setDate(nextDate.getDate() + (7 * 6));
    nextNextDate.setDate(nextNextDate.getDate() + (7 * 6));

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
  model() {
    return { stable: stableReleases() };
  }
});
