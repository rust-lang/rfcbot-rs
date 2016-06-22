import { moduleFor, test } from 'ember-qunit';

moduleFor('controller:issues', 'Unit | Controller | issues', {
  // Specify the other units that are required for this test.
  // needs: ['controller:foo']
});

// Replace this with your real tests.
test('it exists', function(assert) {
  const controller = this.subject();
  assert.ok(controller);
  assert.ok(controller.get('start'));
});
