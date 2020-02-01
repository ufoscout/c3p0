//mod bb8;
mod error;
//mod json;
mod pool;

pub use c3p0_common::*;

pub mod pg {

  //  pub use crate::bb8::*;
  //  pub use crate::json::*;
  ///  pub use crate::pool::*;

    pub mod bb8 {
        pub use bb8::*;
        pub use bb8_postgres::*;
    }
    pub mod driver {
        pub use tokio_postgres::*;
    }
}

#[cfg(test)]
mod test {

    use std::pin::Pin;
    use futures::Stream;
    use std::io;

    async fn jump_around(
        mut stream: Pin<&mut dyn Stream<Item = Result<u8, io::Error>>>,
    ) -> Result<(), io::Error> {
        use futures::stream::TryStreamExt; // for `try_for_each_concurrent`
        const MAX_CONCURRENT_JUMPERS: usize = 100;

        stream.try_for_each_concurrent(MAX_CONCURRENT_JUMPERS, |num| async move {
            //jump_n_times(num).await?;
            //report_n_jumps(num).await?;
            Ok(())
        }).await?;

        Ok(())
    }

}
