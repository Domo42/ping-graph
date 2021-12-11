use super::PingBucket;

#[cfg(test)]
mod tests {
    #[test]
    fn no_ping_lost() {
        // given
        let mut sut = super::PingBucket::new();

        // when
        sut.feed_latency(1);
        sut.feed_latency(5);

        // then
        assert_eq!(sut.lost_percentage(), 0.0, "Expected lost percentage of 0.0%.")
    }

    #[test]
    fn half_pings_lost() {
        // given
        let mut sut = super::PingBucket::new();

        // when
        sut.feed_latency(1);
        sut.feed_lost_ping();
        sut.feed_latency(5);
        sut.feed_lost_ping();

        // then
        assert_eq!(sut.lost_percentage(), 50.0, "Expected lost percentage of 50%.")
    }

    #[test]
    fn calculates_average() {
        // given
        let mut sut = super::PingBucket::new();
        sut.feed_latency(1);
        sut.feed_latency(3);
        sut.feed_latency(5);
        sut.feed_latency(3);

        // when
        assert_eq!(sut.latency_avg, 3.0, "Expected arithmetic average as result.");
    }
}